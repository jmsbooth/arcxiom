//! Unit, interoperability, security, performance and command-line conformance.
//! Independent of the Python compiler. The Python parity job runs separately.
use arcxiom_ir::{compile_str, diff, export, guardrail_delta, query, verify_json};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const PUBLISHED: &str = include_str!("../../../examples/emergency-response.arch.yaml");
const MULTIDOMAIN: &str = include_str!("../../../examples/zero-trust-federation.arch.yaml");
const GOLDEN: &str = include_str!("../../../tests/fixtures/emergency-response.digest");

fn valid(source: &str) -> Value {
    let doc = compile_str(source);
    assert!(doc.valid(), "{:?}", doc.diagnostics);
    doc.ir.expect("valid IR")
}
fn error(source: &str, code: &str) {
    let c = compile_str(source);
    assert!(!c.valid(), "accepted invalid input: {source}");
    assert!(
        c.diagnostics.iter().any(|d| d.code == code),
        "wanted {code}, got {:?}",
        c.diagnostics
    );
}
#[test]
fn published_yaml_compiles_without_python_and_reproduces_digest() {
    let compiled = compile_str(PUBLISHED);
    assert!(compiled.valid(), "{:?}", compiled.diagnostics);
    assert_eq!(compiled.digest().unwrap(), GOLDEN.trim());
    let ir = compiled.ir.unwrap();
    assert_eq!(ir["entities"].as_array().unwrap().len(), 8);
    assert_eq!(ir["relationships"].as_array().unwrap().len(), 7);
    let json = export(&ir).unwrap();
    assert_eq!(verify_json(json.as_bytes()).unwrap(), GOLDEN.trim());
    assert_eq!(
        serde_json::from_str::<Value>(&json).unwrap()["sha256"],
        GOLDEN.trim()
    );
}
#[test]
fn original_multi_domain_crosses_physical_service_data_and_mission_types() {
    let ir = valid(MULTIDOMAIN);
    assert_eq!(ir["namespace"], "zero-trust-federation");
    assert_eq!(ir["constraints"].as_array().unwrap().len(), 1);
    let reach = query(&ir, "role.data-steward", "outgoing", 8).unwrap();
    let ids: Vec<_> = reach["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"mission.share-intelligence"));
    assert!(ids.contains(&"data.mission-report"));
}
#[test]
fn declaration_order_not_hash_relevant() {
    let left="version: 0.1-draft\nnamespace: demo\nentities:\n  - id: role.b\n    type: role\n  - id: role.a\n    type: role\nrelationships:\n  - from: role.a\n    relation: depends_on\n    to: role.b\n";
    let right="namespace: demo\nrelationships:\n  - to: role.b\n    from: role.a\n    relation: depends_on\nentities:\n  - type: role\n    id: role.a\n  - type: role\n    id: role.b\nversion: 0.1-draft\n";
    assert_eq!(
        arcxiom_ir::digest(&valid(left)).unwrap(),
        arcxiom_ir::digest(&valid(right)).unwrap()
    );
    assert!(!diff(&valid(left), &valid(right)).unwrap()["changed"]
        .as_bool()
        .unwrap());
}
#[test]
fn rejects_unsupported_relationship_types_and_domain_range() {
    let head="version: 0.1-draft\nnamespace: r\nentities:\n  - {id: role.a, type: role}\n  - {id: role.b, type: role}\nrelationships:\n";
    error(
        &(head.to_owned() + "  - {from: role.a, relation: carries, to: role.b}\n"),
        "AXR007",
    );
    error(
        &(head.to_owned() + "  - {from: role.a, relation: magic, to: role.b}\n"),
        "AXR003",
    );
    error(
        &(head.to_owned() + "  - {from: role.a, relation: supports, to: absent}\n"),
        "AXR004",
    );
    error(
        &(head.to_owned() + "  - {from: role.a, relation: supports, to: role.b, typo: x}\n"),
        "AXR006",
    );
    error(&(head.to_owned()+"  - {from: role.a, relation: depends_on, to: role.b}\n  - {from: role.a, relation: depends_on, to: role.b}\n"),"AXR005");
}
#[test]
fn data_flow_references_are_typed() {
    let base =
        "version: 0.1-draft\nnamespace: example\nentities:\n - id: flow.x\n   type: data_flow\n";
    error(&(base.to_owned() + "   carries: [missing]\n"), "AXE007");
    error(&(base.to_owned() + "   source: missing\n"), "AXE006");
    error(&(base.to_owned() + "   carries: [flow.x]\n"), "AXE007");
    error(&(base.to_owned() + "   destination: {id: bad}\n"), "AXE006");
}
#[test]
fn security_constraint_is_fail_closed() {
    let baseline = valid(PUBLISHED);
    let unencrypted = PUBLISHED.replace("    transport_encrypted: true\nrelationships:", "    transport_encrypted: false\nrelationships:");
    error(&unencrypted, "AXC005");
    let missing = PUBLISHED.replace(
        "    transport_encrypted: true\nrelationships:",
        "relationships:",
    );
    error(&missing, "AXC004");
    let unused = PUBLISHED.replace("classification: restricted", "classification: public");
    error(&unused, "AXC008");
    let removed = PUBLISHED.replace("enforcement: hard", "enforcement: advisory");
    let diff = guardrail_delta(&baseline, &valid(&removed));
    assert_eq!(diff[0]["code"], "AXG002");
}
#[test]
fn diff_has_exit_code_friendly_stable_semantics() {
    let a = valid(PUBLISHED);
    let changed = PUBLISHED.replace(
        "Coordinate emergency response",
        "Coordinate coordinated response",
    );
    let b = valid(&changed);
    let d = diff(&a, &b).unwrap();
    assert_eq!(d["summary"]["entities"]["modified"], 1);
    assert_eq!(d["changes"].as_array().unwrap().len(), 1);
    assert_eq!(d["changes"][0]["id"], "mission.coordinate-response");
    let second = MULTIDOMAIN;
    assert!(diff(&a, &valid(second)).is_err());
    let no_change = diff(&a, &a).unwrap();
    assert_eq!(no_change["changed"], false);
    assert!(no_change["changes"].as_array().unwrap().is_empty());
}
#[test]
fn malicious_yaml_and_unbounded_inputs() {
    error(
        "version: 0.1-draft\nversion: 0.1-draft\nnamespace: foo\n",
        "AXY002",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: &anchor []\n",
        "AXY003",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: *anchor\n",
        "AXY003",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: !!python/object/apply:os.system [x]\n",
        "AXY003",
    );
    error("version: 0.1-draft\nnamespace: a\nentities: [\n", "AXY001");
    error(
        "version: 0.1-draft\nnamespace: a\nentities: []\n---\nnamespace: b\n",
        "AXY009",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: [1.5]\n",
        "AXY007",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: [18446744073709551616]\n",
        "AXY007",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: !!str []\n",
        "AXY003",
    );
    error(
        "version: 0.1-draft\nnamespace: a\nentities: [ { id: x, id: y } ]\n",
        "AXY002",
    );
    let deep = format!(
        "version: 0.1-draft\nnamespace: a\nentities: {}[]{}\n",
        "[".repeat(60),
        "]".repeat(60)
    );
    error(&deep, "AXY008");
    let huge = format!(
        "version: 0.1-draft\nnamespace: a\nnotes: {}\n",
        "x".repeat(1_048_576)
    );
    error(&huge, "AXY011");
}
#[test]
fn illegal_source_fields_and_collision_checks() {
    error(
        "version: 0.1-draft\nnamespace: a\nunknown: data\n",
        "AXS001",
    );
    error("version: 0.1-draft\nnamespace: A\n", "AXS003");
    error("version: 0.1-draft\nnamespace: a\nentities:\n - {id: role.x, type: role}\n - {id: role.x, type: role}\n","AXE003");
    error(
        "version: 0.1-draft\nnamespace: a\nentities:\n - {id: role.x, type: role, name: ''}\n",
        "AXE005",
    );
    error("version: 0.1-draft\nnamespace: a\nconstraints:\n - id: x\n   select: {type: role}\n   must: {active: true}\n   description: ignored\n","AXC009");
    error("version: 0.1-draft\nnamespace: a\nentities: []\nconstraints:\n - id: x\n   select: {type: role}\n   must: {active: true}\n","AXC008");
}
#[test]
fn forged_or_malformed_ir_never_verifies() {
    let ir = valid(PUBLISHED);
    let payload = export(&ir).unwrap();
    let forged = payload.replace(GOLDEN.trim(), &"0".repeat(64));
    assert!(verify_json(forged.as_bytes())
        .unwrap_err()
        .contains("AXIR19"));
    let dupe=br#"{"archir_version":"0.1-draft","archir_version":"0.1-draft","namespace":"x","entities":[],"relationships":[],"constraints":[]}"#;
    assert!(verify_json(dupe).unwrap_err().contains("AXIR08"));
    let float=br#"{"archir_version":"0.1-draft","namespace":"x","entities":[],"relationships":[],"constraints":[],"extra":1.3}"#;
    assert!(verify_json(float).is_err());
}
#[test]
fn query_direction_bounds_and_cycles() {
    let ir = valid(PUBLISHED);
    assert!(query(&ir, "not.real", "outgoing", 2)
        .unwrap_err()
        .contains("AXQ001"));
    assert!(query(&ir, "role.dispatcher", "sideways", 2)
        .unwrap_err()
        .contains("AXQ002"));
    assert!(query(&ir, "role.dispatcher", "outgoing", 11)
        .unwrap_err()
        .contains("AXQ002"));
    assert_eq!(
        query(&ir, "role.dispatcher", "outgoing", 0).unwrap()["nodes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let reach = query(&ir, "mission.coordinate-response", "incoming", 8).unwrap();
    assert!(reach["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["id"] == "role.dispatcher"));
}
#[test]
fn property_fuzz_randomized_text_never_panics() {
    // Deterministic grammar mutation/property smoke; not a substitute for cargo-fuzz.
    let mut x = 0x13579bdf_u64;
    for _ in 0..256 {
        let mut text = String::from("version: 0.1-draft\nnamespace: smoke\n");
        for _ in 0..64 {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            text.push((32 + (x % 94) as u8) as char);
        }
        let result = compile_str(&text);
        if result.valid() {
            let _ = result.digest().unwrap();
        }
    }
}
#[test]
fn performance_smoke_250_nodes_and_249_edges() {
    let mut src = String::from("version: 0.1-draft\nnamespace: load-test\nentities:\n");
    for i in 0..250 {
        src.push_str(&format!(
            " - {{id: component.{i:03}, type: component, capacity: 18446744073709551615}}\n"
        ));
    }
    src.push_str("relationships:\n");
    for i in 1..250 {
        src.push_str(&format!(
            " - {{from: component.{:03}, relation: depends_on, to: component.{i:03}}}\n",
            i - 1
        ));
    }
    let start = Instant::now();
    let ir = valid(&src);
    let elapsed = start.elapsed();
    assert_eq!(ir["relationships"].as_array().unwrap().len(), 249);
    assert!(
        elapsed < Duration::from_secs(15),
        "250-node compile regression: {elapsed:?}"
    );
}
#[test]
fn canonical_unicode_and_limits() {
    let src="version: 0.1-draft\nnamespace: unicode\nentities:\n - id: role.x\n   type: role\n   name: Café 😀\n";
    let ir = valid(src);
    let json = arcxiom_ir::canonical_json(&ir).unwrap();
    assert!(json.contains(r"\u00e9"));
    assert!(json.contains(r"\ud83d\ude00"));
    let broken = serde_json::json!({"invalid":1.0});
    assert!(arcxiom_ir::canonical_json(&broken).is_err());
    assert!(arcxiom_ir::canonical_json(&json!({"max":u64::MAX})).is_ok());
}
#[test]
fn source_file_read_and_utf8() {
    let temp = std::env::temp_dir().join(format!(
        "arcxiom-{}-{}.arch.yaml",
        std::process::id(),
        "utf8"
    ));
    std::fs::write(&temp, b"\xff\xfe").unwrap();
    let model = arcxiom_ir::compile_file(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(model.diagnostics[0].code, "AXY012");
}
#[cfg(unix)]
#[test]
fn symlink_components_are_rejected() {
    use std::os::unix::fs::symlink;
    let dest = std::env::temp_dir().join(format!("arcxiom-real-{}.arch.yaml", std::process::id()));
    let link = std::env::temp_dir().join(format!("arcxiom-link-{}.arch.yaml", std::process::id()));
    std::fs::write(&dest, PUBLISHED).unwrap();
    symlink(&dest, &link).unwrap();
    let result = arcxiom_ir::compile_file(&link);
    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_file(&dest);
    assert_eq!(result.diagnostics[0].code, "AXY010");
}
