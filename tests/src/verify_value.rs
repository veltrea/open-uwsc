// Standalone verification for RuntimeValue (V3)
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<RuntimeValue>),
    Hash(HashMap<String, RuntimeValue>),
    Null,
    Empty,
    Void,
}

impl RuntimeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Integer(n) => *n != 0,
            RuntimeValue::Float(n) => *n != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::Hash(h) => !h.is_empty(),
            RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => false,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            RuntimeValue::Integer(n) => *n as f64,
            RuntimeValue::Float(n) => *n,
            RuntimeValue::String(s) => s.parse().unwrap_or(0.0),
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RuntimeValue::Integer(n) => n.to_string(),
            RuntimeValue::Float(n) => n.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
            RuntimeValue::Array(_) => "[Array]".into(),
            RuntimeValue::Hash(_) => "{Hash}".into(),
            RuntimeValue::Null => "NULL".into(),
            RuntimeValue::Empty => "".into(),
            RuntimeValue::Void => "".into(),
        }
    }
}

fn main() {
    println!("--- Verifying RuntimeValue (V3) ---");
    
    // Test 1: Implicit Stringify
    if RuntimeValue::Integer(123).to_string() != "123" { panic!("Implicit Stringify failed for Integer"); }
    if RuntimeValue::Float(12.3).to_string() != "12.3" { panic!("Implicit Stringify failed for Float"); }
    if RuntimeValue::Bool(true).to_string() != "TRUE" { panic!("Implicit Stringify failed for Bool"); }
    if RuntimeValue::Empty.to_string() != "" { panic!("Implicit Stringify failed for Empty"); }
    println!("[OK] Implicit Stringify");

    // Test 2: Truthiness
    if !RuntimeValue::Integer(1).is_truthy() { panic!("Truthiness failed for Integer(1)"); }
    if RuntimeValue::Integer(0).is_truthy() { panic!("Truthiness failed for Integer(0)"); }
    if !RuntimeValue::String("Hi".into()).is_truthy() { panic!("Truthiness failed for String"); }
    if RuntimeValue::String("".into()).is_truthy() { panic!("Truthiness failed for empty String"); }
    println!("[OK] Truthiness");

    // Test 3: Coercion to f64
    if RuntimeValue::String("100.5".into()).as_f64() != 100.5 { panic!("Coercion failed for numeric String"); }
    if RuntimeValue::Bool(true).as_f64() != 1.0 { panic!("Coercion failed for Bool(true)"); }
    println!("[OK] Coercion to f64");

    println!("--- All RuntimeValue tests passed ---");
}
