// RuntimeValue::add (V3) Test-First suite
// This script defines the exact behavior expected for the + operator
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

    pub fn as_f64(&self) -> f64 {
        match self {
            RuntimeValue::Integer(n) => *n as f64,
            RuntimeValue::Float(n) => *n,
            RuntimeValue::String(s) => s.parse().unwrap_or(0.0),
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => 0.0,
            _ => 0.0,
        }
    }

    // This is the method we are about to implement. 
    // For now it returns a placeholder to allow the test script to compile.
    pub fn add(&self, _other: &RuntimeValue) -> Result<RuntimeValue, String> {
        Err("Not implemented yet".into())
    }
}

fn main() {
    println!("--- Running RuntimeValue::add Test Suite ---");

    // Case 1: String Concatenation (Priority 1)
    let s10 = RuntimeValue::Integer(10);
    let spt = RuntimeValue::String("pt".into());
    assert_eq!(s10.add(&spt).unwrap(), RuntimeValue::String("10pt".into()));
    
    let sempty = RuntimeValue::Empty;
    let sval = RuntimeValue::String("val".into());
    assert_eq!(sempty.add(&sval).unwrap(), RuntimeValue::String("val".into()));
    println!("[OK] String Concatenation");

    // Case 2: Integer Addition & Overflow
    let one = RuntimeValue::Integer(1);
    let two = RuntimeValue::Integer(2);
    assert_eq!(one.add(&two).unwrap(), RuntimeValue::Integer(3));
    
    let max = RuntimeValue::Integer(i64::MAX);
    let overflow = max.add(&one).unwrap();
    match overflow {
        RuntimeValue::Float(f) => assert_eq!(f, (i64::MAX as f64) + 1.0),
        _ => panic!("Expected Float on Integer overflow"),
    }
    println!("[OK] Integer Addition & Overflow");

    // Case 3: Mixed Numeric (Float propagation)
    let f1_5 = RuntimeValue::Float(1.5);
    let i2 = RuntimeValue::Integer(2);
    assert_eq!(f1_5.add(&i2).unwrap(), RuntimeValue::Float(3.5));
    
    let btrue = RuntimeValue::Bool(true);
    assert_eq!(f1_5.add(&btrue).unwrap(), RuntimeValue::Float(2.5));
    println!("[OK] Mixed Numeric");

    // Case 4: Special Values (Empty/Null as 0)
    assert_eq!(sempty.add(&i2).unwrap(), RuntimeValue::Integer(2));
    let snull = RuntimeValue::Null;
    assert_eq!(snull.add(&i2).unwrap(), RuntimeValue::Integer(2));
    println!("[OK] Special Values as 0");

    // Case 5: Errors (Unsupported types)
    let arr = RuntimeValue::Array(vec![]);
    let hash = RuntimeValue::Hash(HashMap::new());
    assert!(arr.add(&hash).is_err());
    assert!(one.add(&arr).is_err());
    println!("[OK] Error Handling for Unsupported Types");

    println!("--- All tests passed! (Wait, they should fail if not implemented) ---");
}
