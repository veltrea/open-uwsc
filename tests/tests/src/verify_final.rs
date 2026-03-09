// RuntimeValue::add (V3) Verification Suite
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
            RuntimeValue::String(s) => s.parse().unwrap_or(0.0),
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => 0.0,
            _ => 0.0,
        }
    }

    pub fn add(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        if matches!(self, RuntimeValue::String(_)) || matches!(other, RuntimeValue::String(_)) {
            return Ok(RuntimeValue::String(format!("{}{}", self.to_string(), other.to_string())));
        }
        match (self, other) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                if let Some(res) = a.checked_add(*b) {
                    Ok(RuntimeValue::Integer(res))
                } else {
                    Ok(RuntimeValue::Float(*a as f64 + *b as f64))
                }
            }
            (RuntimeValue::Float(a), b) | (b, RuntimeValue::Float(a)) => {
                let res = *a + b.as_f64();
                if res.is_nan() || res.is_infinite() {
                    return Err("ArithmeticError: Result is infinity or NaN".into());
                }
                Ok(RuntimeValue::Float(res))
            }
            (RuntimeValue::Bool(a), RuntimeValue::Integer(b)) => {
                let val = if *a { 1 } else { 0 };
                Ok(RuntimeValue::Integer(val + *b))
            }
            (RuntimeValue::Integer(a), RuntimeValue::Bool(b)) => {
                let val = if *b { 1 } else { 0 };
                Ok(RuntimeValue::Integer(*a + val))
            }
            (RuntimeValue::Empty, b) | (b, RuntimeValue::Empty) |
            (RuntimeValue::Null, b) | (b, RuntimeValue::Null) => {
                match b {
                    RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(*n)),
                    RuntimeValue::Float(n) => Ok(RuntimeValue::Float(*n)),
                    RuntimeValue::Bool(v) => Ok(RuntimeValue::Integer(if *v { 1 } else { 0 })),
                    RuntimeValue::Empty | RuntimeValue::Null => Ok(RuntimeValue::Integer(0)),
                    _ => Err(format!("TypeError: '+' operator cannot be applied to {} and {}", 
                                     self.type_name(), b.type_name())),
                }
            }
            _ => Err(format!("TypeError: '+' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name())),
        }
    }
}

fn main() {
    println!("--- RuntimeValue::add Verification ---");
    
    // Test Vectors
    let cases = vec![
        (RuntimeValue::Integer(1), RuntimeValue::Integer(2), Ok(RuntimeValue::Integer(3))),
        (RuntimeValue::Integer(i64::MAX), RuntimeValue::Integer(1), Ok(RuntimeValue::Float(i64::MAX as f64 + 1.0))),
        (RuntimeValue::String("A".into()), RuntimeValue::Integer(1), Ok(RuntimeValue::String("A1".into()))),
        (RuntimeValue::Empty, RuntimeValue::Integer(10), Ok(RuntimeValue::Integer(10))),
        (RuntimeValue::Null, RuntimeValue::Float(5.5), Ok(RuntimeValue::Float(5.5))),
        (RuntimeValue::Bool(true), RuntimeValue::Integer(1), Ok(RuntimeValue::Integer(2))),
        (RuntimeValue::Float(1.5), RuntimeValue::Integer(2), Ok(RuntimeValue::Float(3.5))),
    ];

    for (l, r, expected) in cases {
        let res = l.add(&r);
        if res != expected {
            panic!("Test Failed: {:?} + {:?} -> Expected {:?}, got {:?}", l, r, expected, res);
        }
    }

    // Error Case
    let arr = RuntimeValue::Array(vec![]);
    assert!(arr.add(&RuntimeValue::Integer(1)).is_err());

    println!("Success: All 7 verification cases passed.");
}
