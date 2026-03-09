use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use serde_json::Value;

pub struct JsonBuiltins;

impl BuiltinModule for JsonBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("JSON_PARSE".into(), json_parse);
        r.insert("JSON_STRINGIFY".into(), json_stringify);
    }
}

fn json_parse(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("JSON_PARSE requires a JSON string".into()); }
    let json_str = args[0].to_string();
    
    let v: Value = serde_json::from_str(&json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    Ok(json_to_runtime(v))
}

fn json_stringify(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("JSON_STRINGIFY requires a value".into()); }
    let val = &args[0];
    let v = runtime_to_json(val);
    
    let result = serde_json::to_string(&v).map_err(|e| format!("JSON stringify failed: {}", e))?;
    Ok(RuntimeValue::String(result))
}

fn json_to_runtime(v: Value) -> RuntimeValue {
    match v {
        Value::Null => RuntimeValue::Null,
        Value::Bool(b) => RuntimeValue::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                RuntimeValue::Integer(i)
            } else {
                RuntimeValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => RuntimeValue::String(s),
        Value::Array(arr) => {
            RuntimeValue::Array(arr.into_iter().map(json_to_runtime).collect())
        }
        Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_runtime(v));
            }
            RuntimeValue::Hash { map, case_care: false }
        }
    }
}

fn runtime_to_json(v: &RuntimeValue) -> Value {
    match v {
        RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => Value::Null,
        RuntimeValue::Bool(b) => Value::Bool(*b),
        RuntimeValue::Integer(i) => Value::Number((*i).into()),
        RuntimeValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Value::Number(n)
            } else {
                Value::Null
            }
        }
        RuntimeValue::String(s) => Value::String(s.clone()),
        RuntimeValue::Array(arr) => {
            Value::Array(arr.iter().map(runtime_to_json).collect())
        }
        RuntimeValue::Hash { map, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), runtime_to_json(v));
            }
            Value::Object(obj)
        }
    }
}
