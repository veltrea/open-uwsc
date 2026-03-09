// RuntimeValue::div (V3) Test-First suite
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
    pub fn div(&self, _other: &RuntimeValue) -> Result<RuntimeValue, String> {
        Err("Not implemented yet".into())
    }
}

fn main() {
    println!("--- Running RuntimeValue::div Test Suite ---");

    // Case 1: Integer Division (Always returns Float)
    let ten = RuntimeValue::Integer(10);
    let two = RuntimeValue::Integer(2);
    assert_eq!(ten.div(&two).unwrap(), RuntimeValue::Float(5.0));
    
    let five = RuntimeValue::Integer(5);
    assert_eq!(five.div(&two).unwrap(), RuntimeValue::Float(2.5));
    println!("[OK] Integer Division -> Float");

    // Case 2: Float Propagation
    let f10 = RuntimeValue::Float(10.0);
    let four = RuntimeValue::Integer(4);
    assert_eq!(f10.div(&four).unwrap(), RuntimeValue::Float(2.5));
    println!("[OK] Float Propagation");

    // Case 3: Special Values as 0
    let sempty = RuntimeValue::Empty;
    assert_eq!(sempty.div(&five).unwrap(), RuntimeValue::Float(0.0));
    
    let btrue = RuntimeValue::Bool(true);
    assert_eq!(btrue.div(&two).unwrap(), RuntimeValue::Float(0.5));
    println!("[OK] Special Values as 0");

    // Case 4: Division by Zero (ArithmeticError)
    assert!(ten.div(&RuntimeValue::Integer(0)).is_err());
    assert!(ten.div(&RuntimeValue::Null).is_err());
    println!("[OK] Division by Zero Check");

    // Case 5: Type Errors
    let s_a = RuntimeValue::String("A".into());
    assert!(s_a.div(&two).is_err());
    println!("[OK] Type Error Check");

    println!("--- All initial tests finished (Should have failed if not implemented) ---");
}
