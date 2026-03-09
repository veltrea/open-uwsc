// RuntimeValue::mod_op (V3) Test-First suite
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

    pub fn as_i64(&self) -> i64 {
        match self {
            RuntimeValue::Integer(n) => *n,
            RuntimeValue::Float(n) => *n as i64,
            RuntimeValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
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
    pub fn mod_op(&self, _other: &RuntimeValue) -> Result<RuntimeValue, String> {
        Err("Not implemented yet".into())
    }
}

fn main() {
    println!("--- Running RuntimeValue::mod_op Test Suite ---");

    // Case 1: Standard Integer Modulo
    let ten = RuntimeValue::Integer(10);
    let three = RuntimeValue::Integer(3);
    assert_eq!(ten.mod_op(&three).unwrap(), RuntimeValue::Integer(1));
    println!("[OK] Standard Integer Modulo");

    // Case 2: Float Propagation
    let f10_5 = RuntimeValue::Float(10.5);
    let res = f10_5.mod_op(&three).unwrap();
    assert_eq!(res.as_f64(), 1.5);
    println!("[OK] Float Propagation");

    // Case 3: Boolean handling
    let btrue = RuntimeValue::Bool(true);
    let two = RuntimeValue::Integer(2);
    assert_eq!(btrue.mod_op(&two).unwrap(), RuntimeValue::Integer(1));
    println!("[OK] Boolean handling");

    // Case 4: Division by Zero (ArithmeticError)
    assert!(ten.mod_op(&RuntimeValue::Integer(0)).is_err());
    let snull = RuntimeValue::Null;
    assert!(ten.mod_op(&snull).is_err());
    println!("[OK] Modulo by Zero Check");

    // Case 5: Type Errors
    let s_a = RuntimeValue::String("A".into());
    assert!(s_a.mod_op(&three).is_err());
    println!("[OK] Type Error Check");

    println!("--- All initial tests finished (Should have failed if not implemented) ---");
}
