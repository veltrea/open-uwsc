// RuntimeValue::cmp_op (V3) Test-First suite
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

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Eq, Ne, Gt, Lt, Ge, Le
}

impl RuntimeValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Integer(_) => "Integer",
            RuntimeValue::Float(_) => "Float",
            RuntimeValue::String(_) => "String",
            RuntimeValue::Bool(_) => "Boolean",
            RuntimeValue::Array(_) => "Array",
            RuntimeValue::Hash(_) => "Hash",
            RuntimeValue::Null => "Null",
            RuntimeValue::Empty => "Empty",
            RuntimeValue::Void => "Void",
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            RuntimeValue::Integer(n) => *n as f64,
            RuntimeValue::Float(n) => *n,
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => 0.0,
            RuntimeValue::String(s) => s.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn is_numeric_compat(&self) -> bool {
        matches!(self, 
            RuntimeValue::Integer(_) | 
            RuntimeValue::Float(_) | 
            RuntimeValue::Bool(_) | 
            RuntimeValue::Empty | 
            RuntimeValue::Null)
    }

    // --- TO BE IMPLEMENTED ---
    pub fn compare(&self, other: &RuntimeValue, op: CmpOp) -> Result<RuntimeValue, String> {
        Err("Not implemented yet".into())
    }
}

fn main() {
    println!("--- Running RuntimeValue::compare Test Suite ---");

    // Case 1: Standard Numeric Equality
    let ten = RuntimeValue::Integer(10);
    let f10 = RuntimeValue::Float(10.0);
    assert_eq!(ten.compare(&f10, CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
    println!("[OK] Numeric Equality (Int == Float)");

    // Case 2: String Case-Insensitive Equality
    let abc1 = RuntimeValue::String("abc".into());
    let abc2 = RuntimeValue::String("ABC".into());
    assert_eq!(abc1.compare(&abc2, CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
    assert_eq!(abc1.compare(&abc2, CmpOp::Ne).unwrap(), RuntimeValue::Bool(false));
    println!("[OK] String Case-Insensitive Equality");

    // Case 3: Numeric vs String Comparison (Implicit coercion)
    let s10 = RuntimeValue::String("10".into());
    assert_eq!(ten.compare(&s10, CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
    println!("[OK] Numeric vs String Equality");

    // Case 4: Inequality operators
    let five = RuntimeValue::Integer(5);
    assert_eq!(ten.compare(&five, CmpOp::Gt).unwrap(), RuntimeValue::Bool(true));
    assert_eq!(five.compare(&ten, CmpOp::Le).unwrap(), RuntimeValue::Bool(true));
    println!("[OK] Inequality operators (>, <=)");

    // Case 5: Special values (Empty/Null as 0)
    let sempty = RuntimeValue::Empty;
    let zero = RuntimeValue::Integer(0);
    assert_eq!(sempty.compare(&zero, CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
    println!("[OK] Special values as 0");

    println!("--- All initial tests finished (Should have failed if not implemented) ---");
}
