use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use regex::Regex;

pub struct StringExtBuiltins;

impl BuiltinModule for StringExtBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("RE_MATCH".into(), re_match);
        r.insert("RE_FIND".into(), re_find);
        r.insert("RE_REPLACE".into(), re_replace);
    }
}

fn re_match(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("RE_MATCH requires 2 arguments (pattern, text)".into()); }
    let pattern = args[0].to_string();
    let text = args[1].to_string();
    
    let re = Regex::new(&pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    Ok(RuntimeValue::Bool(re.is_match(&text)))
}

fn re_find(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("RE_FIND requires 2 arguments (pattern, text)".into()); }
    let pattern = args[0].to_string();
    let text = args[1].to_string();
    
    let re = Regex::new(&pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    let matches: Vec<RuntimeValue> = re.find_iter(&text)
        .map(|m| RuntimeValue::String(m.as_str().to_string()))
        .collect();
    
    Ok(RuntimeValue::Array(matches))
}

fn re_replace(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("RE_REPLACE requires 3 arguments (pattern, replacement, text)".into()); }
    let pattern = args[0].to_string();
    let replacement = args[1].to_string();
    let text = args[2].to_string();
    
    let re = Regex::new(&pattern).map_err(|e| format!("Invalid regex: {}", e))?;
    let result = re.replace_all(&text, replacement.as_str()).to_string();
    
    Ok(RuntimeValue::String(result))
}
