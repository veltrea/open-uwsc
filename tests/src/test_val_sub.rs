// RuntimeValue::sub (V3) Verification Suite
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

    pub fn sub(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        match (self, other) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                if let Some(res) = a.checked_sub(*b) {
                    Ok(RuntimeValue::Integer(res))
                } else {
                    Ok(RuntimeValue::Float(*a as f64 - *b as f64))
                }
            }
            (RuntimeValue::Float(a), b) | (b, RuntimeValue::Float(a)) => {
                let val = match self {
                    RuntimeValue::Float(f) => *f - other.as_f64(),
                    _ => self.as_f64() - other.as_f64(),
                };
                if val.is_nan() || val.is_infinite() {
                    return Err("ArithmeticError: Result is infinity or NaN".into());
                }
                Ok(RuntimeValue::Float(val))
            }
            (RuntimeValue::Bool(a), RuntimeValue::Integer(b)) => {
                let val = if *a { 1 } else { 0 };
                Ok(RuntimeValue::Integer(val - *b))
            }
            (RuntimeValue::Integer(a), RuntimeValue::Bool(b)) => {
                let val = if *b { 1 } else { 0 };
                Ok(RuntimeValue::Integer(*a - val))
            }
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => {
                let v1 = if *a { 1 } else { 0 };
                let v2 = if *b { 1 } else { 0 };
                Ok(RuntimeValue::Integer(v1 - v2))
            }
            (RuntimeValue::Empty, b) | (b, RuntimeValue::Empty) |
            (RuntimeValue::Null, b) | (b, RuntimeValue::Null) => {
                let v1 = self.as_f64();
                let v2 = b.as_f64();
                Ok(RuntimeValue::Integer((v1 - v2) as i64))
            }
            _ => Err(format!("TypeError: '-' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name())),
        }
    }
}

fn main() {
    println!("--- Verifying RuntimeValue::sub ---");

    macro_rules! test_case {
        ($l:expr, $r:expr, $expected_val:expr, $name:expr) => {
            print!("Testing {}: ... ", $name);
            let res = $l.sub(&$r);
            let expected: Result<RuntimeValue, String> = Ok($expected_val);
            if res == expected {
                println!("OK");
            } else {
                println!("FAILED");
                println!("  Left: {:?}, Right: {:?}", $l, $r);
                println!("  Expected: {:?}, Got: {:?}", expected, res);
                std::process::exit(1);
            }
        };
    }

    let five = RuntimeValue::Integer(5);
    let three = RuntimeValue::Integer(3);
    test_case!(five, three, RuntimeValue::Integer(2), "Standard Int");

    let min = RuntimeValue::Integer(i64::MIN);
    let one = RuntimeValue::Integer(1);
    test_case!(min, one, RuntimeValue::Float((i64::MIN as f64) - 1.0), "Underflow Promotion");

    let f5_5 = RuntimeValue::Float(5.5);
    let i2 = RuntimeValue::Integer(2);
    test_case!(f5_5, i2, RuntimeValue::Float(3.5), "Float Mixed");
    
    let btrue = RuntimeValue::Bool(true);
    test_case!(btrue, one, RuntimeValue::Integer(0), "Bool-Int");

    let sempty = RuntimeValue::Empty;
    let i10 = RuntimeValue::Integer(10);
    test_case!(sempty, i10, RuntimeValue::Integer(-10), "Empty-Int");

    let snull = RuntimeValue::Null;
    test_case!(i10, snull, RuntimeValue::Integer(10), "Int-Null");

    println!("SUCCESS: All sub tests passed!");
}
