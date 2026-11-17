//! Experimental Rust checker for normalized Arcxiom 0.1 ArchIR, not a PDP.
use serde_json::Value;
use sha2::{Digest,Sha256};
use std::{collections::HashSet,env,fs,process};

fn field<'a>(v:&'a Value,k:&str)->Result<&'a str,String>{
    v.get(k).and_then(Value::as_str).ok_or(format!("AXIR01: invalid {k}"))
}
fn list<'a>(v:&'a Value,k:&str,max:usize)->Result<&'a Vec<Value>,String>{
    let a=v.get(k).and_then(Value::as_array).ok_or(format!("AXIR02: invalid {k}"))?;
    if a.len()>max{return Err(format!("AXIR03: too many {k}"));}
    Ok(a)
}
fn shape(v:&Value,keys:&[&str])->Result<(),String>{
    let m=v.as_object().ok_or("AXIR04: expected object")?;
    if m.len()!=keys.len()||m.keys().any(|k|!keys.contains(&k.as_str())){
        return Err("AXIR05: unexpected or missing fields".into());
    }
    Ok(())
}
fn ident(s:&str)->bool{
    !s.is_empty()&&s.len()<=128&&s.chars().next().is_some_and(|c|c.is_ascii_alphabetic())
        &&s.chars().all(|c|c.is_ascii_alphanumeric()||"._-".contains(c))
}
fn canon(v:&Value)->Result<String,String>{
    match v{
        Value::Null=>Ok("null".into()),
        Value::Bool(b)=>Ok(b.to_string()),
        Value::Number(n) if n.is_i64()||n.is_u64()=>Ok(n.to_string()),
        Value::Number(_)=>Err("AXIR06: float canonicalization is not defined".into()),
        Value::String(s)=>{
            let mut out=String::new();
            for c in serde_json::to_string(s).map_err(|e|e.to_string())?.chars(){
                if c.is_ascii(){out.push(c);}else{
                    let mut buf=[0u16;2];
                    for unit in c.encode_utf16(&mut buf).iter(){
                        out.push_str(&format!("\\u{unit:04x}"));
                    }
                }
            }
            Ok(out)
        }
        Value::Array(items)=>Ok(format!("[{}]",items.iter().map(canon)
            .collect::<Result<Vec<_>,_>>()?.join(","))),
        Value::Object(items)=>{
            let pairs=items.iter().map(|(k,v)|
                Ok(format!("{}:{}",canon(&Value::String(k.clone()))?,canon(v)?)))
                .collect::<Result<Vec<_>,String>>()?;
            Ok(format!("{{{}}}",pairs.join(",")))
        }
    }
}
fn verify(bytes:&[u8])->Result<String,String>{
    if bytes.len()>4*1024*1024{return Err("AXIR07: input too large".into());}
    let mut ir:Value=serde_json::from_slice(bytes).map_err(|e|format!("AXIR08: {e}"))?;
    let claimed=ir.get("sha256").map(|v|v.as_str().ok_or("AXIR09: sha256 must be text"))
        .transpose()?.map(str::to_owned);
    ir.as_object_mut().ok_or("AXIR04: expected object")?.remove("sha256");
    shape(&ir,&["archir_version","namespace","entities","relationships","constraints"])?;
    if field(&ir,"archir_version")?!="0.1-draft"{return Err("AXIR10: wrong version".into());}
    let ns=field(&ir,"namespace")?;
    if ns.len()>64||ns.is_empty()||!ns.chars().next().is_some_and(|c|c.is_ascii_lowercase())
        ||!ns.chars().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||"-_".contains(c)){
        return Err("AXIR11: invalid namespace".into());
    }
    let mut ids=HashSet::new();
    for entity in list(&ir,"entities",1000)?{
        shape(entity,&["id","type","name","attributes"])?;
        let id=field(entity,"id")?;
        if !ident(id)||!ids.insert(id.to_owned()){return Err("AXIR12: duplicate/invalid ID".into());}
        if field(entity,"type")?.is_empty()||field(entity,"name")?.is_empty()
            ||!entity["attributes"].is_object(){return Err("AXIR13: invalid entity".into());}
    }
    let mut edges=HashSet::new();
    for edge in list(&ir,"relationships",4000)?{
        shape(edge,&["from","relation","to"])?;
        let a=field(edge,"from")?;let b=field(edge,"to")?;let r=field(edge,"relation")?;
        if !ids.contains(a)||!ids.contains(b)||r.is_empty(){return Err("AXIR14: invalid edge".into());}
        if !edges.insert((a.to_owned(),r.to_owned(),b.to_owned())){
            return Err("AXIR15: duplicate edge".into());
        }
    }
    let mut rules=HashSet::new();
    for rule in list(&ir,"constraints",500)?{
        shape(rule,&["id","select","must","missing_evidence","enforcement"])?;
        let id=field(rule,"id")?;
        if !ident(id)||!rules.insert(id.to_owned()){return Err("AXIR16: duplicate/invalid rule".into());}
        if rule["select"].as_object().is_none_or(|m|m.is_empty())||
            rule["must"].as_object().is_none_or(|m|m.is_empty()){
            return Err("AXIR17: empty rule".into());
        }
        if !["fail","warn"].contains(&field(rule,"missing_evidence")?)||
            !["hard","advisory"].contains(&field(rule,"enforcement")?){
            return Err("AXIR18: invalid rule mode".into());
        }
    }
    let digest=format!("{:x}",Sha256::digest(canon(&ir)?.as_bytes()));
    if claimed.is_some_and(|v|v!=digest){return Err("AXIR19: digest mismatch".into());}
    Ok(digest)
}
fn main(){
    let args:Vec<String>=env::args().collect();
    if args.len()!=3||args[1]!="verify"{
        eprintln!("usage: arcxiom-ir verify <archir.json>");process::exit(64);
    }
    let data=fs::read(&args[2]).unwrap_or_else(|e|{
        eprintln!("AXIR20: {e}");process::exit(2)
    });
    match verify(&data){
        Ok(hash)=>println!("sha256:{hash}"),
        Err(err)=>{eprintln!("{err}");process::exit(2);}
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn unicode_matches_python(){
        assert_eq!(canon(&serde_json::json!({"x":"é😀"})).unwrap(),
            "{\"x\":\"\\u00e9\\ud83d\\ude00\"}");
    }
    #[test]
    fn forged_digest_rejected(){
        let bad=br#"{"archir_version":"0.1-draft","namespace":"x","entities":[],"relationships":[],"constraints":[],"sha256":"bad"}"#;
        assert!(verify(bad).unwrap_err().contains("AXIR19"));
    }
    #[test]
    fn missing_endpoint_rejected(){
        let bad=br#"{"archir_version":"0.1-draft","namespace":"x","entities":[],"relationships":[{"from":"x","relation":"supports","to":"y"}],"constraints":[]}"#;
        assert!(verify(bad).unwrap_err().contains("AXIR14"));
    }
}