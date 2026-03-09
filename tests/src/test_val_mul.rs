// RuntimeValue::mul (V3) Test-First suite
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

    // --- TO BE IMPLEMENTED ---
    pub fn mul(&self, _other: &RuntimeValue) -> Result<RuntimeValue, String> {
        Err("Not implemented yet".into())
    }
}

fn main() {
    println!("--- Running RuntimeValue::mul Test Suite ---");

    // Case 1: Standard Integer Multiplication
    let five = RuntimeValue::Integer(5);
    let three = RuntimeValue::Integer(3);
    assert_eq!(five.mul(&three).unwrap(), RuntimeValue::Integer(15));
    println!("[OK] Standard Integer Multiplication");

    // Case 2: Integer Overflow (Promotion to Float)
    let big = RuntimeValue::Integer(1_000_000_000);
    let huge = RuntimeValue::Integer(1_000_000_000_000);
    let res = big.mul(&huge).unwrap();
    match res {
        RuntimeValue::Float(f) => assert_eq!(f, 1.0e21),
        _ => panic!("Expected Float on Integer overflow, got {:?}", res),
    }
    println!("[OK] Integer Overflow Promotion");

    // Case 3: Mixed Numeric
    let two = RuntimeValue::Integer(2);
    let f2_5 = RuntimeValue::Float(2.5);
    assert_eq!(two.mul(&f2_5).unwrap(), RuntimeValue::Float(5.0));
    println!("[OK] Mixed Numeric");

    // Case 4: Special Values
    let sempty = RuntimeValue::Empty;
    let i10 = RuntimeValue::Integer(10);
    assert_eq!(sempty.mul(&i10).unwrap(), RuntimeValue::Integer(0));
    
    let btrue = RuntimeValue::Bool(true);
    assert_eq!(btrue.mul(&five).unwrap(), RuntimeValue::Integer(5));
    println!("[OK] Special Values");

    // Case 5: Errors
    let s_a = RuntimeValue::String("A".into());
    assert!(s_a.mul(&five).is_err());
    println!("[OK] Error Handling");

    println!("--- All tests passed! ---");
}
