//! Portable Arcxiom CLI. Rust is the authoritative source compiler.
use arcxiom_ir::{compile_file, diff, export, guardrail_delta, query, verify_json, Compiled};
use serde_json::json;
use std::{env, fs, path::Path, process};

fn usage() -> String {
    "usage: arcxiom <validate|export|query|diff|verify> [path(s)] [options]\n\
     validate FILE [--json]\n\
     export FILE [--format json] [-o|--output FILE]\n\
     query FILE --id ID [--direction incoming|outgoing|both] [--depth 0..10] [--json]\n\
     diff BASELINE CANDIDATE [--check-guardrails] [--json]\n\
     verify ARCHIR.json\n\
     exit codes: 0 success/identical, 2 invalid source/guardrail, 3 changes, 64 usage".into()
}
#[derive(Default)]
struct Options {
    json: bool,
    format: Option<String>,
    output: Option<String>,
    id: Option<String>,
    direction: Option<String>,
    depth: Option<usize>,
    guardrails: bool,
    positional: Vec<String>,
}
fn args_parse(args: &[String]) -> Result<Options,String> {
    let mut o = Options::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--json" => o.json = true,
            "--check-guardrails" => o.guardrails = true,
            "--format" | "--output" | "-o" | "--id" | "--direction" | "--depth" => {
                i += 1;
                let value = args.get(i).ok_or_else(||format!("missing value for {arg}"))?.clone();
                match arg {
                    "--format" => o.format = Some(value),
                    "--output" | "-o" => o.output = Some(value),
                    "--id" => o.id = Some(value),
                    "--direction" => o.direction = Some(value),
                    "--depth" => o.depth = Some(value.parse().map_err(|_|"invalid depth".to_string())?),
                    _ => unreachable!(),
                }
            }
            "--help" | "-h" => return Err(usage()),
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
            _ => o.positional.push(arg.to_string()),
        }
        i += 1;
    }
    Ok(o)
}
fn report(model:&Compiled, as_json:bool) {
    if as_json {
        let payload=json!({"valid":model.valid(),"digest":model.digest().ok(),"diagnostics":model.diagnostics});
        println!("{}",serde_json::to_string_pretty(&payload).expect("serializable diagnostics"));
    } else {
        for d in &model.diagnostics {
            eprintln!("{} {} {}: {}",d.severity.to_uppercase(),d.code,d.path,d.message);
        }
        if model.valid() {
            let ir=model.ir.as_ref().expect("validated");
            println!("valid  |  {} entities  |  {} relationships  |  sha256:{}",
                ir["entities"].as_array().map_or(0,Vec::len),
                ir["relationships"].as_array().map_or(0,Vec::len),
                model.digest().expect("canonical validation"));
        }
    }
}
fn qualified_input(path:&str)->Result<Compiled, String> {
    let c=compile_file(Path::new(path));
    if c.valid() { Ok(c) } else {
        for d in &c.diagnostics { eprintln!("{} {} {}: {}",d.severity.to_uppercase(),d.code,d.path,d.message); }
        Err(format!("AXS999: rejected architecture source: {path}"))
    }
}
fn main_run(args: &[String]) -> Result<i32,(i32,String)> {
    if args.is_empty() { return Err((64, usage())); }
    if args[0] == "--version" || args[0] == "-V" {
        println!("arcxiom {} | ArchIR {}",env!("CARGO_PKG_VERSION"),arcxiom_ir::VERSION);
        return Ok(0);
    }
    if args[0] == "--help" || args[0] == "-h" {
        println!("{}",usage());
        return Ok(0);
    }
    let opts=args_parse(&args[1..]).map_err(|e|(64,e))?;
    match args[0].as_str() {
        "validate" => {
            if opts.positional.len()!=1 {return Err((64,usage()));}
            let model=compile_file(Path::new(&opts.positional[0]));
            report(&model,opts.json);
            Ok(if model.valid(){0}else{2})
        }
        "export" => {
            if opts.positional.len()!=1 || opts.json || opts.format.as_deref().is_some_and(|f|f!="json") {
                return Err((64,usage()));
            }
            let c=qualified_input(&opts.positional[0]).map_err(|e|(2,e))?;
            let payload=export(c.ir.as_ref().expect("validated")).map_err(|e|(2,e))?;
            if let Some(path)=opts.output {
                let target=Path::new(&path);
                if target.is_symlink() || !target.parent().is_some_and(|p|p.is_dir()) {
                    return Err((2,"AXO001: symlink or missing output directory".into()));
                }
                fs::write(target,payload.as_bytes()).map_err(|e|(2,format!("AXO002: {e}")))?;
                println!("{path}");
            } else {print!("{payload}");}
            Ok(0)
        }
        "query" => {
            if opts.positional.len()!=1 || opts.id.is_none() {return Err((64,usage()));}
            let c=qualified_input(&opts.positional[0]).map_err(|e|(2,e))?;
            let start=opts.id.as_deref().expect("checked id");
            let direction=opts.direction.as_deref().unwrap_or("outgoing");
            let depth=opts.depth.unwrap_or(3);
            let result=query(c.ir.as_ref().expect("validated"),start,direction,depth).map_err(|e|(2,e))?;
            if opts.json {println!("{}",serde_json::to_string_pretty(&result).expect("serializable query"));}
            else {
                for node in result["nodes"].as_array().expect("typed query") {
                    println!("{}  {:<20} {}",node["distance"],node["type"].as_str().unwrap_or(""),node["id"].as_str().unwrap_or(""));
                }
            }
            Ok(0)
        }
        "diff" => {
            if opts.positional.len()!=2 {return Err((64,usage()));}
            let before=qualified_input(&opts.positional[0]).map_err(|e|(2,e))?;
            let after=qualified_input(&opts.positional[1]).map_err(|e|(2,e))?;
            let a=before.ir.as_ref().expect("validated");
            let b=after.ir.as_ref().expect("validated");
            let mut delta=diff(a,b).map_err(|e|(2,e))?;
            let violations=if opts.guardrails {guardrail_delta(a,b)}else{vec![]};
            delta["guardrail_violations"]=json!(violations);
            if opts.json {println!("{}",serde_json::to_string_pretty(&delta).expect("serializable diff"));}
            else {
                for change in delta["changes"].as_array().expect("typed diff") {
                    println!("{}  {:<9} {}",change["section"].as_str().unwrap_or(""),
                        change["action"].as_str().unwrap_or(""),change["id"].as_str().unwrap_or(""));
                }
                if delta["changed"]==false {println!("no semantic changes");}
                for v in &violations {eprintln!("ERROR {} {}: {}",v["code"],v["object_id"],v["message"]);}
            }
            Ok(if !violations.is_empty(){2}else if delta["changed"]==true{3}else{0})
        }
        "verify" => {
            if opts.positional.len()!=1 {return Err((64,usage()));}
            let data=fs::read(&opts.positional[0]).map_err(|e|(2,format!("AXIR20: {e}")))?;
            let hash=verify_json(&data).map_err(|e|(2,e))?;
            println!("sha256:{hash}");
            Ok(0)
        }
        _ => Err((64,usage()))
    }
}
fn main() {
    let args:Vec<String>=env::args().skip(1).collect();
    match main_run(&args) {
        Ok(code)=>process::exit(code),
        Err((code,message))=>{eprintln!("{message}");process::exit(code);}
    }
}
