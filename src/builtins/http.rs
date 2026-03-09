use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use reqwest::blocking::Client;

pub struct HttpBuiltins;

impl BuiltinModule for HttpBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("HTTP_GET".into(), http_get);
        r.insert("HTTP_POST".into(), http_post);
    }
}

fn http_get(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("HTTP_GET requires a URL".into()); }
    let url = args[0].to_string();
    
    let client = Client::new();
    let resp = client.get(&url).send()
        .map_err(|e| format!("HTTP GET failed: {}", e))?;
    
    let body = resp.text().map_err(|e| format!("Failed to read response body: {}", e))?;
    Ok(RuntimeValue::String(body))
}

fn http_post(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("HTTP_POST requires a URL and a body (JSON string)".into()); }
    let url = args[0].to_string();
    let body_json = args[1].to_string();
    
    let client = Client::new();
    let resp = client.post(&url)
        .header("Content-Type", "application/json")
        .body(body_json)
        .send()
        .map_err(|e| format!("HTTP POST failed: {}", e))?;
    
    let body = resp.text().map_err(|e| format!("Failed to read response body: {}", e))?;
    Ok(RuntimeValue::String(body))
}
