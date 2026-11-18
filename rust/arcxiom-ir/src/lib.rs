//! Arcxiom 0.1-draft architecture compiler and deterministic graph kernel.
//! Experimental architecture assertions, not evidence of runtime security.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::path::Path;

pub const VERSION: &str = "0.1-draft";
pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_DEPTH: usize = 20;
pub const MAX_ENTITIES: usize = 1000;
pub const MAX_RELATIONSHIPS: usize = 4000;
pub const MAX_CONSTRAINTS: usize = 500;
const TYPES: &[&str] = &[
    "mission",
    "objective",
    "capability",
    "organization",
    "actor",
    "role",
    "business_process",
    "business_service",
    "system",
    "system_of_systems",
    "subsystem",
    "software",
    "application",
    "service",
    "component",
    "api",
    "interface",
    "data_asset",
    "data_flow",
    "event",
    "network",
    "hardware",
    "device",
    "facility",
    "location",
    "requirement",
    "control",
    "risk",
    "decision",
];
const RELATIONS: &[&str] = &[
    "contains",
    "realizes",
    "supports",
    "implements",
    "depends_on",
    "operates",
    "invokes",
    "connects_to",
    "deployed_on",
    "governs",
    "carries",
    "flows_to",
    "verified_by",
    "assigned_to",
    "exchanges_with",
];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub path: String,
}
impl Diagnostic {
    pub fn error(code: &str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity: "error".into(),
            message: message.into(),
            path: path.into(),
        }
    }
    pub fn warning(code: &str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            severity: "warning".into(),
            ..Self::error(code, message, path)
        }
    }
}
#[derive(Clone, Debug)]
pub struct Compiled {
    pub ir: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
}
impl Compiled {
    pub fn valid(&self) -> bool {
        self.ir.is_some() && !self.diagnostics.iter().any(|d| d.severity == "error")
    }
    pub fn digest(&self) -> Result<String, String> {
        digest(self.ir.as_ref().ok_or("cannot digest invalid source")?)
    }
}
fn fail(d: Diagnostic) -> Compiled {
    Compiled {
        ir: None,
        diagnostics: vec![d],
    }
}
fn ordered(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_by(|a, b| {
        (a.severity != "error", &a.path, &a.code, &a.message).cmp(&(
            b.severity != "error",
            &b.path,
            &b.code,
            &b.message,
        ))
    });
    diagnostics
}
fn ident(s: &str) -> bool {
    s.len() <= 128
        && s.bytes().next().is_some_and(|b| b.is_ascii_alphabetic())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn namespace(s: &str) -> bool {
    s.len() <= 64
        && s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}
fn field<'a>(obj: &'a Value, k: &str) -> Option<&'a str> {
    obj.get(k).and_then(Value::as_str)
}
fn mapping(v: &Value) -> Option<&Map<String, Value>> {
    v.as_object()
}
fn unsupported(obj: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    obj.keys()
        .filter(|k| !keys.contains(&k.as_str()))
        .cloned()
        .collect()
}
fn opt_array<'a>(
    obj: &'a Map<String, Value>,
    key: &str,
    ds: &mut Vec<Diagnostic>,
    max: usize,
) -> Option<&'a [Value]> {
    match obj.get(key) {
        None => Some(&[]),
        Some(Value::Array(v)) if v.len() <= max => Some(v),
        Some(Value::Array(_)) => {
            ds.push(Diagnostic::error(
                "AXS005",
                format!("too many {key} (max {max})"),
                key,
            ));
            None
        }
        _ => {
            ds.push(Diagnostic::error(
                "AXS004",
                format!("{key} must be an array"),
                key,
            ));
            None
        }
    }
}

// A strict serde visitor detects duplicate mapping keys at every level before any
// JSON object construction. serde_json::Value alone would silently replace keys.
struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Visit;
        impl<'de> Visitor<'de> for Visit {
            type Value = Strict;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("bounded JSON-compatible YAML")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Strict, E> {
                Ok(Strict(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Strict, E> {
                Err(E::custom("floating-point values are unsupported"))
            }
            fn visit_i128<E: de::Error>(self, _: i128) -> Result<Strict, E> {
                Err(E::custom("integer exceeds i64/u64 profile"))
            }
            fn visit_u128<E: de::Error>(self, _: u128) -> Result<Strict, E> {
                Err(E::custom("integer exceeds i64/u64 profile"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Strict, E> {
                Ok(Strict(Value::String(v.to_owned())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Strict, E> {
                Ok(Strict(Value::String(v)))
            }
            fn visit_none<E: de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_some<D: Deserializer<'de>>(self, de: D) -> Result<Strict, D::Error> {
                Strict::deserialize(de)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Strict, A::Error> {
                let mut items = Vec::new();
                while let Some(value) = seq.next_element::<Strict>()? {
                    items.push(value.0);
                    if items.len() > MAX_RELATIONSHIPS + MAX_ENTITIES + MAX_CONSTRAINTS {
                        return Err(de::Error::custom("sequence item limit exceeded"));
                    }
                }
                Ok(Strict(Value::Array(items)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Strict, A::Error> {
                let mut items = Map::new();
                while let Some((key, value)) = map.next_entry::<String, Strict>()? {
                    if items.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate YAML key: {key}")));
                    }
                    if items.len() >= 4096 {
                        return Err(de::Error::custom("mapping item limit exceeded"));
                    }
                    items.insert(key, value.0);
                }
                Ok(Strict(Value::Object(items)))
            }
        }
        de.deserialize_any(Visit)
    }
}

// Cheap, bounded lexical rejection before recursive YAML deserialization. Restrict
// the supported alpha syntax instead of trying to sanitize general YAML after
// aliases/tags/merge expansion. Quoted strings and comments are not scanned as tokens.
fn preflight(src: &str) -> Result<(), Diagnostic> {
    if src.len() > MAX_BYTES {
        return Err(Diagnostic::error("AXY011", "input exceeds 1 MiB", "source"));
    }
    if src.as_bytes().contains(&0) {
        return Err(Diagnostic::error(
            "AXY001",
            "NUL bytes are not allowed",
            "source",
        ));
    }
    let mut indents: Vec<usize> = vec![0];
    let mut inline_depth = 0usize;
    let mut quote: Option<char> = None;
    for (n, raw) in src.lines().enumerate() {
        let trimmed = raw.trim();
        if matches!(trimmed, "---" | "...")
            || trimmed.starts_with("--- ")
            || trimmed.starts_with("... ")
        {
            return Err(Diagnostic::error(
                "AXY009",
                "document separators are unsupported",
                format!("line:{}", n + 1),
            ));
        }
        let indent = raw.bytes().take_while(|c| *c == b' ').count();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && quote.is_none() && inline_depth == 0
        {
            while indents.last().copied().is_some_and(|x| indent < x) {
                indents.pop();
            }
            if indent > *indents.last().unwrap_or(&0) {
                indents.push(indent);
            }
            if indents.len() > MAX_DEPTH + 1 {
                return Err(Diagnostic::error(
                    "AXY008",
                    "YAML nesting limit exceeded",
                    format!("line:{}", n + 1),
                ));
            }
        }
        let mut previous = ' ';
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if let Some(q) = quote {
                if q == '"' && c == '\\' {
                    chars.next();
                    continue;
                }
                if c == q {
                    if q == '\'' && chars.peek() == Some(&'\'') {
                        chars.next();
                    } else {
                        quote = None;
                    }
                }
                continue;
            }
            if c == '#' && (previous.is_whitespace() || matches!(previous, '[' | '{' | ',')) {
                break;
            }
            if c == '"' || c == '\'' {
                quote = Some(c);
                previous = c;
                continue;
            }
            let boundary =
                previous.is_whitespace() || matches!(previous, ':' | '[' | '{' | ',' | '-');
            if boundary && matches!(c, '&' | '*' | '!') && chars.peek().is_some() {
                return Err(Diagnostic::error(
                    "AXY003",
                    "YAML anchors, aliases and tags are unsupported",
                    format!("line:{}", n + 1),
                ));
            }
            if matches!(c, '[' | '{') {
                inline_depth += 1;
            }
            if matches!(c, ']' | '}') {
                inline_depth = inline_depth.saturating_sub(1);
            }
            if inline_depth + indents.len().saturating_sub(1) > MAX_DEPTH {
                return Err(Diagnostic::error(
                    "AXY008",
                    "YAML nesting limit exceeded",
                    format!("line:{}", n + 1),
                ));
            }
            previous = c;
        }
        // YAML block scalars are deliberately excluded in this compatibility subset.
        let line = raw.split(" #").next().unwrap_or(raw).trim_end();
        if line.ends_with(": |")
            || line.ends_with(": >")
            || line.ends_with(": |-")
            || line.ends_with(": >-")
        {
            return Err(Diagnostic::error(
                "AXY001",
                "block scalars are not supported",
                format!("line:{}", n + 1),
            ));
        }
    }
    Ok(())
}

pub fn compile_str(src: &str) -> Compiled {
    if let Err(d) = preflight(src) {
        return fail(d);
    }
    // Exact safe-Rust parser behavior is part of the 0.1-draft compatibility profile.
    // Do not expand YAML merges or collapse duplicate keys before our visitor can
    // reject them. Preserve ambiguous numeric spellings rather than coercing them.
    let parser_options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::PassThrough,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error,
        strict_booleans: true,
        leading_zero_numbers_as_strings: true,
        underscored_numbers_as_strings: true,
    };
    let raw = match serde_saphyr::from_str_with_options::<Strict>(src, parser_options) {
        Ok(v) => v.0,
        Err(e) => {
            let msg = e.to_string();
            let code = if msg.contains("duplicate YAML key") {
                "AXY002"
            } else if msg.contains("integer") || msg.contains("floating-point") {
                "AXY007"
            } else {
                "AXY001"
            };
            return fail(Diagnostic::error(code, msg, "source"));
        }
    };
    compile_document(&raw)
}

pub fn compile_file(path: &Path) -> Compiled {
    // Reject symlink path components, not only a symlink at the final filename.
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|m| m.file_type().is_symlink()) {
            return fail(Diagnostic::error(
                "AXY010",
                "symlink input path is unsupported",
                path.display().to_string(),
            ));
        }
    }
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            return fail(Diagnostic::error(
                "AXY012",
                format!("cannot read source: {e}"),
                path.display().to_string(),
            ))
        }
    };
    if data.len() > MAX_BYTES {
        return fail(Diagnostic::error(
            "AXY011",
            "input exceeds 1 MiB",
            path.display().to_string(),
        ));
    }
    match String::from_utf8(data) {
        Ok(src) => compile_str(&src),
        Err(e) => fail(Diagnostic::error(
            "AXY012",
            format!("source is not UTF-8: {e}"),
            path.display().to_string(),
        )),
    }
}

// Validate public JSON values before interpreting semantics, not only after YAML parsing.
fn bounded_json(value: &Value, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => true,
        Value::Number(n) => n.is_i64() || n.is_u64(),
        Value::Array(a) => {
            a.len() <= MAX_ENTITIES + MAX_RELATIONSHIPS + MAX_CONSTRAINTS
                && a.iter().all(|v| bounded_json(v, depth + 1))
        }
        Value::Object(m) => {
            m.len() <= 4096 && m.values().all(|v| bounded_json(v, depth + 1))
        }
    }
}

pub fn compile_document(root: &Value) -> Compiled {
    if !bounded_json(root, 0) {
        return fail(Diagnostic::error(
            "AXS006",
            "unsupported scalar, numeric range/float, count or nesting depth",
            "root",
        ));
    }
    let mut ds = Vec::new();
    let Some(obj) = root.as_object() else {
        return fail(Diagnostic::error(
            "AXS000",
            "document must be a mapping",
            "root",
        ));
    };
    for key in unsupported(
        obj,
        &[
            "version",
            "namespace",
            "entities",
            "relationships",
            "constraints",
        ],
    ) {
        ds.push(Diagnostic::error(
            "AXS001",
            format!("unsupported top-level field: {key}"),
            key,
        ));
    }
    if field(root, "version") != Some(VERSION) {
        ds.push(Diagnostic::error(
            "AXS002",
            format!("expected version {VERSION}"),
            "version",
        ));
    }
    let ns = field(root, "namespace");
    if !ns.is_some_and(namespace) {
        ds.push(Diagnostic::error(
            "AXS003",
            "invalid namespace",
            "namespace",
        ));
    }
    let entities = opt_array(obj, "entities", &mut ds, MAX_ENTITIES);
    let relations = opt_array(obj, "relationships", &mut ds, MAX_RELATIONSHIPS);
    let rules = opt_array(obj, "constraints", &mut ds, MAX_CONSTRAINTS);
    let (Some(entities), Some(relations), Some(rules)) = (entities, relations, rules) else {
        return Compiled {
            ir: None,
            diagnostics: ordered(ds),
        };
    };
    let mut nodes = BTreeMap::<String, Value>::new();
    for (i, node) in entities.iter().enumerate() {
        let loc = format!("entities[{i}]");
        let Some(fields) = mapping(node) else {
            ds.push(Diagnostic::error("AXE001", "entity must be a map", loc));
            continue;
        };
        let id = field(node, "id");
        if !id.is_some_and(ident) {
            ds.push(Diagnostic::error(
                "AXE002",
                "invalid entity id",
                format!("{loc}.id"),
            ));
            continue;
        }
        let id = id.expect("checked id");
        if nodes.contains_key(id) {
            ds.push(Diagnostic::error(
                "AXE003",
                format!("duplicate id: {id}"),
                format!("{loc}.id"),
            ));
            continue;
        }
        let kind = field(node, "type");
        if !kind.is_some_and(|t| TYPES.contains(&t)) {
            ds.push(Diagnostic::error(
                "AXE004",
                "unsupported entity type",
                format!("{loc}.type"),
            ));
            continue;
        }
        let name = if fields.contains_key("name") {
            field(node, "name")
        } else {
            Some(id)
        };
        if !name.is_some_and(|s| !s.trim().is_empty() && s.chars().count() <= 256) {
            ds.push(Diagnostic::error(
                "AXE005",
                "invalid entity name",
                format!("{loc}.name"),
            ));
            continue;
        }
        let mut attrs = fields.clone();
        attrs.remove("id");
        attrs.remove("type");
        attrs.remove("name");
        nodes.insert(
            id.to_owned(),
            json!({"id":id,"type":kind,"name":name,"attributes":attrs}),
        );
    }
    let mut edges = BTreeMap::<(String, String, String), Value>::new();
    for (i, edge) in relations.iter().enumerate() {
        let loc = format!("relationships[{i}]");
        let Some(fields) = mapping(edge) else {
            ds.push(Diagnostic::error(
                "AXR001",
                "relationship must be a map",
                loc,
            ));
            continue;
        };
        let extra = unsupported(fields, &["from", "relation", "to"]);
        if !extra.is_empty() {
            ds.push(Diagnostic::error(
                "AXR006",
                format!("unsupported relationship fields: {}", extra.join(", ")),
                loc,
            ));
            continue;
        }
        let (Some(source), Some(rel), Some(target)) = (
            field(edge, "from"),
            field(edge, "relation"),
            field(edge, "to"),
        ) else {
            ds.push(Diagnostic::error(
                "AXR002",
                "from, relation and to must be strings",
                loc,
            ));
            continue;
        };
        if !RELATIONS.contains(&rel) {
            ds.push(Diagnostic::error(
                "AXR003",
                format!("unsupported relationship: {rel}"),
                format!("{loc}.relation"),
            ));
            continue;
        }
        let (Some(from), Some(to)) = (nodes.get(source), nodes.get(target)) else {
            ds.push(Diagnostic::error(
                "AXR004",
                format!("unknown endpoint: {source} -> {target}"),
                loc,
            ));
            continue;
        };
        let a = field(from, "type").unwrap_or("");
        let b = field(to, "type").unwrap_or("");
        let typed = match rel {
            "carries" => a == "data_flow" && b == "data_asset",
            "flows_to" => a == "data_flow" || b == "data_flow",
            "operates" => ["actor", "role", "organization"].contains(&a),
            "implements" => ["service", "api", "application", "software", "component"].contains(&a),
            "realizes" => b == "capability",
            "supports" => ["mission", "objective", "capability"].contains(&b),
            _ => true,
        };
        if !typed {
            ds.push(Diagnostic::error(
                "AXR007",
                format!("incompatible {rel} domain/range: {a} -> {b}"),
                loc,
            ));
            continue;
        }
        let key = (source.to_owned(), rel.to_owned(), target.to_owned());
        if edges.contains_key(&key) {
            ds.push(Diagnostic::error("AXR005", "duplicate relationship", loc));
            continue;
        }
        edges.insert(key, json!({"from":source,"relation":rel,"to":target}));
    }
    let mut normalized_rules = BTreeMap::<String, Value>::new();
    for (i, rule) in rules.iter().enumerate() {
        let loc = format!("constraints[{i}]");
        let Some(fields) = mapping(rule) else {
            ds.push(Diagnostic::error("AXC001", "constraint must be a map", loc));
            continue;
        };
        let id = field(rule, "id");
        if !id.is_some_and(ident) || id.is_some_and(|v| normalized_rules.contains_key(v)) {
            ds.push(Diagnostic::error(
                "AXC002",
                "invalid or duplicate constraint id",
                format!("{loc}.id"),
            ));
            continue;
        }
        let id = id.expect("checked rule id");
        if !unsupported(
            fields,
            &["id", "select", "must", "missing_evidence", "enforcement"],
        )
        .is_empty()
        {
            ds.push(Diagnostic::error(
                "AXC009",
                "unsupported constraint keys (including description)",
                loc,
            ));
            continue;
        }
        let (Some(selector), Some(must)) = (
            rule.get("select").and_then(Value::as_object),
            rule.get("must").and_then(Value::as_object),
        ) else {
            ds.push(Diagnostic::error(
                "AXC003",
                "select and must must be maps",
                loc,
            ));
            continue;
        };
        if selector.is_empty() || must.is_empty() {
            ds.push(Diagnostic::error(
                "AXC003",
                "select and must must be nonempty",
                loc,
            ));
            continue;
        }
        if !unsupported(selector, &["type", "carries_classification"]).is_empty()
            || !selector
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|t| TYPES.contains(&t))
            || selector
                .get("carries_classification")
                .is_some_and(|v| !v.is_string())
        {
            ds.push(Diagnostic::error(
                "AXC006",
                "invalid select type or selector keys",
                format!("{loc}.select"),
            ));
            continue;
        }
        if selector.contains_key("carries_classification")
            && selector.get("type").and_then(Value::as_str) != Some("data_flow")
        {
            ds.push(Diagnostic::error(
                "AXC006",
                "carries_classification requires data_flow selection",
                format!("{loc}.select"),
            ));
            continue;
        }
        let missing = if fields.contains_key("missing_evidence") {
            field(rule, "missing_evidence")
        } else {
            Some("fail")
        };
        let enforcement = if fields.contains_key("enforcement") {
            field(rule, "enforcement")
        } else {
            Some("hard")
        };
        if !missing.is_some_and(|v| ["fail", "warn"].contains(&v))
            || !enforcement.is_some_and(|v| ["hard", "advisory"].contains(&v))
        {
            ds.push(Diagnostic::error(
                "AXC007",
                "invalid missing_evidence or enforcement",
                loc,
            ));
            continue;
        }
        normalized_rules.insert(
            id.to_owned(),
            json!({
                "id":id,"select":selector,"must":must,
                "missing_evidence":missing,"enforcement":enforcement
            }),
        );
    }
    for (id, node) in &nodes {
        if field(node, "type") != Some("data_flow") {
            continue;
        }
        let attrs = &node["attributes"];
        for k in ["source", "destination"] {
            if let Some(value) = attrs.get(k) {
                if !value.as_str().is_some_and(|s| nodes.contains_key(s)) {
                    ds.push(Diagnostic::error(
                        "AXE006",
                        format!("invalid {k} reference"),
                        format!("{id}.{k}"),
                    ));
                }
            }
        }
        if let Some(carried) = attrs.get("carries") {
            if !carried.as_array().is_some_and(|a| {
                a.iter().all(|v| {
                    v.as_str()
                        .and_then(|s| nodes.get(s))
                        .is_some_and(|dest| field(dest, "type") == Some("data_asset"))
                })
            }) {
                ds.push(Diagnostic::error(
                    "AXE007",
                    "carries must reference existing data assets",
                    format!("{id}.carries"),
                ));
            }
        }
    }
    // Constraints operate on asserted architecture attributes, not observed evidence.
    for (id, rule) in &normalized_rules {
        let select = &rule["select"];
        let missing = field(rule, "missing_evidence") == Some("fail");
        let hard = field(rule, "enforcement") == Some("hard");
        let mut selected: Vec<(&String, &Value)> = Vec::new();
        for (eid, entity) in &nodes {
            if field(entity, "type") != field(select, "type") {
                continue;
            }
            if let Some(classification) = field(select, "carries_classification") {
                let matches = entity["attributes"]["carries"].as_array().is_some_and(|a| {
                    a.iter().any(|v| {
                        v.as_str()
                            .and_then(|name| nodes.get(name))
                            .is_some_and(|asset| {
                                field(&asset["attributes"], "classification")
                                    == Some(classification)
                            })
                    })
                });
                if !matches {
                    continue;
                }
            }
            selected.push((eid, entity));
        }
        if selected.is_empty() {
            let message = format!("constraint {id} selected no entities");
            ds.push(if hard && missing {
                Diagnostic::error("AXC008", message, id)
            } else {
                Diagnostic::warning("AXC008", message, id)
            });
        }
        for (eid, entity) in selected {
            for (key, expected) in rule["must"].as_object().expect("validated must") {
                let actual = entity["attributes"].get(key);
                if actual.is_none() {
                    let message = format!("{eid} missing required attribute {key} for {id}");
                    ds.push(if hard && missing {
                        Diagnostic::error("AXC004", message, eid)
                    } else {
                        Diagnostic::warning("AXC004", message, eid)
                    });
                } else if actual != Some(expected) {
                    let message = format!("{eid}.{key} fails constraint {id}");
                    ds.push(if hard {
                        Diagnostic::error("AXC005", message, eid)
                    } else {
                        Diagnostic::warning("AXC005", message, eid)
                    });
                }
            }
        }
    }
    ds = ordered(ds);
    if ds.iter().any(|d| d.severity == "error") {
        return Compiled {
            ir: None,
            diagnostics: ds,
        };
    }
    Compiled {
        ir: Some(json!({
            "archir_version":VERSION,"namespace":ns.expect("validated namespace"),
            "entities":nodes.into_values().collect::<Vec<_>>(),
            "relationships":edges.into_values().collect::<Vec<_>>(),
            "constraints":normalized_rules.into_values().collect::<Vec<_>>()
        })),
        diagnostics: ds,
    }
}

// The canonicalization profile is intentionally the v0.1 Python ensure_ascii
// sorted-key compact JSON profile. Floating point and unsupported numbers fail.
pub fn canonical_json(v: &Value) -> Result<String, String> {
    match v {
        Value::Null => Ok("null".into()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) if n.is_i64() || n.is_u64() => Ok(n.to_string()),
        Value::Number(_) => Err("AXIR06: non-integer JSON number".into()),
        Value::String(s) => {
            let mut out = String::new();
            for c in serde_json::to_string(s).map_err(|e| e.to_string())?.chars() {
                if c.is_ascii() {
                    out.push(c);
                } else {
                    let mut utf16 = [0u16; 2];
                    for unit in c.encode_utf16(&mut utf16).iter() {
                        out.push_str(&format!("\\u{unit:04x}"));
                    }
                }
            }
            Ok(out)
        }
        Value::Array(a) => Ok(format!(
            "[{}]",
            a.iter()
                .map(canonical_json)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Value::Object(m) => Ok(format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| {
                    Ok(format!(
                        "{}:{}",
                        canonical_json(&Value::String(k.clone()))?,
                        canonical_json(v)?
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
                .join(",")
        )),
    }
}
pub fn digest(ir: &Value) -> Result<String, String> {
    let mut plain = ir.clone();
    plain
        .as_object_mut()
        .ok_or("AXIR04: expected object")?
        .remove("sha256");
    Ok(format!(
        "{:x}",
        Sha256::digest(canonical_json(&plain)?.as_bytes())
    ))
}
pub fn export(ir: &Value) -> Result<String, String> {
    let mut value = ir.clone();
    let hash = digest(&value)?;
    value
        .as_object_mut()
        .ok_or("AXIR04: expected object")?
        .insert("sha256".into(), json!(hash));
    serde_json::to_string_pretty(&value)
        .map(|s| format!("{s}\n"))
        .map_err(|e| e.to_string())
}
pub fn verify_json(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() > 4 * MAX_BYTES {
        return Err("AXIR07: ArchIR too large".into());
    }
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let mut ir = Strict::deserialize(&mut de)
        .map_err(|e| format!("AXIR08: {e}"))?
        .0;
    de.end().map_err(|e| format!("AXIR08: {e}"))?;
    let claimed = match ir.get("sha256") {
        None => None,
        Some(Value::String(s)) => Some(s.clone()),
        _ => return Err("AXIR09: sha256 must be text".into()),
    };
    let obj = ir.as_object_mut().ok_or("AXIR04: expected object")?;
    obj.remove("sha256");
    if obj.len() != 5
        || !unsupported(
            obj,
            &[
                "archir_version",
                "namespace",
                "entities",
                "relationships",
                "constraints",
            ],
        )
        .is_empty()
    {
        return Err("AXIR05: unsupported or missing root keys".into());
    }
    if field(&ir, "archir_version") != Some(VERSION)
        || !field(&ir, "namespace").is_some_and(namespace)
    {
        return Err("AXIR10: wrong version or namespace".into());
    }
    let mut ids = HashSet::new();
    for entity in ir["entities"]
        .as_array()
        .filter(|a| a.len() <= MAX_ENTITIES)
        .ok_or("AXIR02: invalid entities")?
    {
        let map = entity.as_object().ok_or("AXIR13: entity must be object")?;
        if map.len() != 4 || !unsupported(map, &["id", "type", "name", "attributes"]).is_empty() {
            return Err("AXIR13: entity shape".into());
        }
        let id = field(entity, "id").ok_or("AXIR12: missing id")?;
        if !ident(id)
            || !ids.insert(id.to_owned())
            || !field(entity, "type").is_some_and(|t| TYPES.contains(&t))
            || field(entity, "name").is_none_or(|n| n.is_empty())
            || !entity["attributes"].is_object()
        {
            return Err("AXIR13: invalid entity".into());
        }
    }
    let mut edges = HashSet::new();
    for edge in ir["relationships"]
        .as_array()
        .filter(|a| a.len() <= MAX_RELATIONSHIPS)
        .ok_or("AXIR02: invalid relationships")?
    {
        let map = edge.as_object().ok_or("AXIR14: edge shape")?;
        if map.len() != 3 || !unsupported(map, &["from", "relation", "to"]).is_empty() {
            return Err("AXIR14: edge shape".into());
        }
        let (Some(source), Some(rel), Some(target)) = (
            field(edge, "from"),
            field(edge, "relation"),
            field(edge, "to"),
        ) else {
            return Err("AXIR14: edge fields".into());
        };
        if !ids.contains(source)
            || !ids.contains(target)
            || !RELATIONS.contains(&rel)
            || !edges.insert((source, rel, target))
        {
            return Err("AXIR14: invalid or duplicate edge".into());
        }
    }
    let mut rules = HashSet::new();
    for rule in ir["constraints"]
        .as_array()
        .filter(|a| a.len() <= MAX_CONSTRAINTS)
        .ok_or("AXIR02: invalid constraints")?
    {
        let map = rule.as_object().ok_or("AXIR17: constraint shape")?;
        if map.len() != 5
            || !unsupported(
                map,
                &["id", "select", "must", "missing_evidence", "enforcement"],
            )
            .is_empty()
        {
            return Err("AXIR17: constraint shape".into());
        }
        let id = field(rule, "id").ok_or("AXIR16: missing rule id")?;
        if !ident(id)
            || !rules.insert(id)
            || rule["select"].as_object().is_none_or(|v| v.is_empty())
            || rule["must"].as_object().is_none_or(|v| v.is_empty())
            || !field(rule, "missing_evidence").is_some_and(|v| ["fail", "warn"].contains(&v))
            || !field(rule, "enforcement").is_some_and(|v| ["hard", "advisory"].contains(&v))
        {
            return Err("AXIR17: invalid constraint".into());
        }
    }
    let computed = digest(&ir)?;
    if claimed.is_some_and(|v| v != computed) {
        return Err("AXIR19: digest mismatch".into());
    }
    // Verify semantic consistency and normalization, not merely a self-asserted
    // checksum. Reconstruct the source document from the strict interchange
    // shape and ask the authoritative compiler to validate it again.
    let mut source_nodes = Vec::new();
    for entity in ir["entities"].as_array().ok_or("AXIR02: missing entities")? {
        let mut node = entity["attributes"]
            .as_object()
            .ok_or("AXIR13: invalid attributes")?
            .clone();
        for key in ["id", "type", "name"] {
            if node.contains_key(key) {
                return Err("AXIR21: reserved entity attribute key".into());
            }
            node.insert(key.into(), entity[key].clone());
        }
        source_nodes.push(Value::Object(node));
    }
    let source = json!({
        "version": VERSION,
        "namespace": ir["namespace"],
        "entities": source_nodes,
        "relationships": ir["relationships"],
        "constraints": ir["constraints"],
    });
    let checked = compile_document(&source);
    if !checked.valid() {
        let first = checked.diagnostics.first().ok_or("AXIR21: rejected semantic graph")?;
        return Err(format!("AXIR21: {} {}: {}", first.code, first.path, first.message));
    }
    if checked.ir.as_ref() != Some(&ir) {
        return Err("AXIR22: noncanonical entity, relationship or constraint ordering".into());
    }
    Ok(computed)
}

pub fn query(ir: &Value, start: &str, direction: &str, depth: usize) -> Result<Value, String> {
    if !["incoming", "outgoing", "both"].contains(&direction) || depth > 10 {
        return Err("AXQ002: invalid direction/depth".into());
    }
    let nodes: BTreeMap<&str, &Value> = ir["entities"]
        .as_array()
        .ok_or("AXQ003: missing entities")?
        .iter()
        .filter_map(|v| field(v, "id").map(|id| (id, v)))
        .collect();
    if !nodes.contains_key(start) {
        return Err(format!("AXQ001: unknown entity id: {start}"));
    }
    let mut seen = BTreeMap::<String, usize>::new();
    seen.insert(start.into(), 0);
    let mut queue = VecDeque::from([start.to_owned()]);
    let mut edges = BTreeSet::<(String, String, String)>::new();
    while let Some(current) = queue.pop_front() {
        let dist = seen[&current];
        if dist >= depth {
            continue;
        }
        for e in ir["relationships"]
            .as_array()
            .ok_or("AXQ003: missing relationships")?
        {
            let a = field(e, "from").unwrap_or("");
            let b = field(e, "to").unwrap_or("");
            let r = field(e, "relation").unwrap_or("");
            let neighbor = if (direction == "outgoing" || direction == "both") && a == current {
                Some(b)
            } else if (direction == "incoming" || direction == "both") && b == current {
                Some(a)
            } else {
                None
            };
            if let Some(next) = neighbor {
                edges.insert((a.into(), r.into(), b.into()));
                if !seen.contains_key(next) {
                    seen.insert(next.into(), dist + 1);
                    queue.push_back(next.into());
                }
            }
        }
    }
    let matches: Vec<_> = seen
        .into_iter()
        .map(|(id, distance)| {
            let node = nodes.get(id.as_str()).expect("query node must exist");
            json!({"id":id,"type":field(node,"type"),"distance":distance})
        })
        .collect();
    let relationships: Vec<_> = edges
        .into_iter()
        .map(|(a, r, b)| json!({"from":a,"relation":r,"to":b}))
        .collect();
    Ok(
        json!({"start":start,"direction":direction,"depth":depth,"nodes":matches,"relationships":relationships}),
    )
}
fn key_for(section: &str, item: &Value) -> String {
    if section == "relationships" {
        format!(
            "{}:{}:{}",
            field(item, "from").unwrap_or(""),
            field(item, "relation").unwrap_or(""),
            field(item, "to").unwrap_or("")
        )
    } else {
        field(item, "id").unwrap_or("").into()
    }
}
pub fn diff(before: &Value, after: &Value) -> Result<Value, String> {
    if before["archir_version"] != after["archir_version"] {
        return Err("AXD001: incompatible ArchIR version".into());
    }
    if before["namespace"] != after["namespace"] {
        return Err("AXD001: incompatible namespaces".into());
    }
    let mut changes = Vec::new();
    let mut summary = Map::new();
    for section in ["entities", "relationships", "constraints"] {
        let left: BTreeMap<_, _> = before[section]
            .as_array()
            .ok_or("AXD001: missing section")?
            .iter()
            .map(|v| (key_for(section, v), v))
            .collect();
        let right: BTreeMap<_, _> = after[section]
            .as_array()
            .ok_or("AXD001: missing section")?
            .iter()
            .map(|v| (key_for(section, v), v))
            .collect();
        let mut added = 0;
        let mut removed = 0;
        let mut modified = 0;
        for key in left.keys().chain(right.keys()).collect::<BTreeSet<_>>() {
            let a = left.get(key);
            let b = right.get(key);
            let action = match (a, b) {
                (None, Some(_)) => {
                    added += 1;
                    "added"
                }
                (Some(_), None) => {
                    removed += 1;
                    "removed"
                }
                (Some(x), Some(y)) if x != y => {
                    modified += 1;
                    "modified"
                }
                _ => continue,
            };
            let mut rec = json!({"section":section,"id":key,"action":action});
            if let Some(v) = a {
                rec["before"] = (*v).clone();
            }
            if let Some(v) = b {
                rec["after"] = (*v).clone();
            }
            changes.push(rec);
        }
        summary.insert(
            section.into(),
            json!({"added":added,"removed":removed,"modified":modified}),
        );
    }
    Ok(
        json!({"archir_version":before["archir_version"],"namespace":before["namespace"],"changed":!changes.is_empty(),"summary":summary,"changes":changes}),
    )
}
pub fn guardrail_delta(before: &Value, after: &Value) -> Vec<Value> {
    let old_rules: BTreeMap<_, _> = before["constraints"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| field(r, "id").map(|id| (id, r)))
        .collect();
    let new_rules: BTreeMap<_, _> = after["constraints"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| field(r, "id").map(|id| (id, r)))
        .collect();
    let old_nodes: BTreeMap<_, _> = before["entities"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| field(r, "id").map(|id| (id, r)))
        .collect();
    let new_nodes: BTreeMap<_, _> = after["entities"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| field(r, "id").map(|id| (id, r)))
        .collect();
    let mut violations = Vec::new();
    for (id, rule) in old_rules {
        if field(rule, "enforcement") != Some("hard") {
            continue;
        }
        let (code, message) = match new_rules.get(id) {
            None => ("AXG001", "approved hard constraint removed"),
            Some(r) if *r != rule => ("AXG002", "approved hard constraint modified"),
            _ => continue,
        };
        violations.push(json!({"code":code,"object_id":id,"message":message}));
    }
    for (id, node) in old_nodes {
        if field(node, "type") != Some("data_asset")
            || node["attributes"].get("classification").is_none()
        {
            continue;
        }
        let (code, message) = match new_nodes.get(id) {
            None => ("AXG003", "classified asset removed; review required"),
            Some(n)
                if field(n, "type") != Some("data_asset")
                    || n["attributes"].get("classification")
                        != node["attributes"].get("classification") =>
            {
                ("AXG004", "classification changed; review required")
            }
            _ => continue,
        };
        violations.push(json!({"code":code,"object_id":id,"message":message}));
    }
    violations
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_unicode_parity() {
        assert_eq!(
            canonical_json(&json!({"x":"é😀"})).unwrap(),
            r#"{"x":"\u00e9\ud83d\ude00"}"#
        );
    }
    #[test]
    fn duplicate_nested_key() {
        let result = compile_str(
            "version: 0.1-draft\nnamespace: a\nentities:\n - id: x\n   id: y\n   type: role\n",
        );
        assert_eq!(result.diagnostics[0].code, "AXY002");
    }
    #[test]
    fn rejects_anchors_and_tags() {
        for src in [
            "version: 0.1-draft\nnamespace: x\nentities: &a []\n",
            "version: 0.1-draft\nnamespace: x\nentities: !!str []\n",
            "version: 0.1-draft\nnamespace: x\nentities: *a\n",
        ] {
            assert_eq!(compile_str(src).diagnostics[0].code, "AXY003");
        }
    }
    #[test]
    fn strict_yaml12_bool() {
        let c=compile_str("version: 0.1-draft\nnamespace: x\nentities:\n - id: role.a\n   type: role\n   on: yes\n   enabled: true\n");
        assert!(c.valid(), "{:?}", c.diagnostics);
        assert_eq!(
            c.ir.unwrap()["entities"][0]["attributes"],
            json!({"on":"yes","enabled":true})
        );
    }
    #[test]
    fn numeric_boundaries() {
        for n in ["-9223372036854775808", "0", "18446744073709551615"] {
            let src=format!("version: 0.1-draft\nnamespace: x\nentities:\n - id: role.a\n   type: role\n   seats: {n}\n");
            assert!(compile_str(&src).valid(), "{n}");
        }
        for n in [
            "18446744073709551616",
            "-9223372036854775809",
            "1.5",
            "1e30",
        ] {
            let src=format!("version: 0.1-draft\nnamespace: x\nentities:\n - id: role.a\n   type: role\n   seats: {n}\n");
            assert!(!compile_str(&src).valid(), "{n}");
        }
    }
}
