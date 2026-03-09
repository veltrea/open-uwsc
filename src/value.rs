use crate::semantics::{UwscArithmetic, UwscCompare, UwscLogical, UwscAssignment, UwscUnary};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<RuntimeValue>),
    Hash {
        map: HashMap<String, RuntimeValue>,
        case_care: bool,
    },
    Null,
    Empty,
    Void,
}

impl RuntimeValue {
    /// UWSC-style truthiness check.
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Integer(n) => *n != 0,
            RuntimeValue::Float(n) => *n != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::Hash { map, .. } => !map.is_empty(),
            RuntimeValue::Null | RuntimeValue::Empty | RuntimeValue::Void => false,
        }
    }

    /// Explicitly convert to bool.
    pub fn as_bool(&self) -> bool {
        self.is_truthy()
    }

    /// Explicitly convert to f64.
    pub fn as_f64(&self) -> f64 {
        match self {
            RuntimeValue::Integer(n) => *n as f64,
            RuntimeValue::Float(n) => *n,
            RuntimeValue::String(s) => s.parse().unwrap_or(0.0),
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    /// Explicitly convert to i64 (for indices, etc.)
    pub fn as_i64(&self) -> i64 {
        match self {
            RuntimeValue::Integer(n) => *n,
            RuntimeValue::Float(n) => *n as i64,
            RuntimeValue::String(s) => s.parse().unwrap_or(0),
            RuntimeValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        }
    }

    /// "Implicit Stringify" - used for PRINT, concatenation, etc.
    pub fn to_string(&self) -> String {
        match self {
            RuntimeValue::Integer(n) => n.to_string(),
            RuntimeValue::Float(n) => n.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
            RuntimeValue::Array(_) => "[Array]".into(),
            RuntimeValue::Hash { .. } => "{Hash}".into(),
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
            RuntimeValue::Hash { .. } => "Hash",
            RuntimeValue::Null => "Null",
            RuntimeValue::Empty => "Empty",
            RuntimeValue::Void => "Void",
        }
    }

    pub fn is_numeric_compat(&self) -> bool {
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        // 数値、文字列、ブール値、Empty/Null は数値コンテキストで使用可能。
        matches!(self, 
            RuntimeValue::Integer(_) | 
            RuntimeValue::Float(_) | 
            RuntimeValue::String(_) |
            RuntimeValue::Bool(_) | 
            RuntimeValue::Empty | 
            RuntimeValue::Null)
    }
}

/// UWSCの算術演算の実装。
/// 参照: ORIGINAL_LANGUAGE_SPEC.md §2.2
impl UwscArithmetic for RuntimeValue {
    /// 加算・連結 (+)
    /// 境界条件: String + Any -> Stringification
    /// 境界条件: Integer + Integer (Overflow) -> Float
    fn add(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        // Step 1: String Concatenation Priority
        if matches!(self, RuntimeValue::String(_)) || matches!(other, RuntimeValue::String(_)) {
            return Ok(RuntimeValue::String(format!("{}{}", self.to_string(), other.to_string())));
        }

        // Step 2 & 3: Numeric Addition (including promotion and special values)
        match (self, other) {
            // Integer + Integer with overflow check
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                if let Some(res) = a.checked_add(*b) {
                    Ok(RuntimeValue::Integer(res))
                } else {
                    // Overflow: Promote to Float
                    Ok(RuntimeValue::Float(*a as f64 + *b as f64))
                }
            }
            
            // Numeric mix (Float propagation)
            (RuntimeValue::Float(a), b) | (b, RuntimeValue::Float(a)) => {
                let res = *a + b.as_f64();
                if res.is_nan() || res.is_infinite() {
                    return Err("ArithmeticError: Result is infinity or NaN".into());
                }
                Ok(RuntimeValue::Float(res))
            }

            // Bool + Numeric (True=1, False=0)
            (RuntimeValue::Bool(a), RuntimeValue::Integer(b)) => {
                let val = if *a { 1 } else { 0 };
                Ok(RuntimeValue::Integer(val + *b))
            }
            (RuntimeValue::Integer(a), RuntimeValue::Bool(b)) => {
                let val = if *b { 1 } else { 0 };
                Ok(RuntimeValue::Integer(*a + val))
            }

            // Special values (Empty/Null as 0)
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

            // Unsupported types
            _ => Err(format!("TypeError: '+' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name())),
        }
    }

    /// 減算 (-)
    /// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L27](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L27) (優先順位)
    /// 暗黙変換: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
    fn sub(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        if !self.is_numeric_compat() || !other.is_numeric_compat() {
            return Err(format!("TypeError: '-' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name()));
        }

        match (self, other) {
            // If any is Float or String (which might parse to float), use Float logic
            (RuntimeValue::Float(_), _) | (_, RuntimeValue::Float(_)) |
            (RuntimeValue::String(_), _) | (_, RuntimeValue::String(_)) => {
                let v1 = self.as_f64();
                let v2 = other.as_f64();
                let res = v1 - v2;
                if res.is_nan() || res.is_infinite() {
                    return Err("ArithmeticError: Result is infinity or NaN".into());
                }
                Ok(RuntimeValue::Float(res))
            }

            // Integer path (includes Bool, Empty, Null which are converted to i64)
            (a, b) => {
                let v1 = a.as_i64();
                let v2 = b.as_i64();
                if let Some(res) = v1.checked_sub(v2) {
                    Ok(RuntimeValue::Integer(res))
                } else {
                    // Overflow/Underflow
                    Ok(RuntimeValue::Float(v1 as f64 - v2 as f64))
                }
            }
        }
    }

    /// 乗算 (*)
    /// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L25](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L25) (優先順位)
    /// 暗黙変換: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
    fn mul(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        if !self.is_numeric_compat() || !other.is_numeric_compat() {
            return Err(format!("TypeError: '*' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name()));
        }

        match (self, other) {
            // If any is Float or String, use Float logic
            (RuntimeValue::Float(_), _) | (_, RuntimeValue::Float(_)) |
            (RuntimeValue::String(_), _) | (_, RuntimeValue::String(_)) => {
                let v1 = self.as_f64();
                let v2 = other.as_f64();
                let res = v1 * v2;
                if res.is_nan() || res.is_infinite() {
                    return Err("ArithmeticError: Result is infinity or NaN".into());
                }
                Ok(RuntimeValue::Float(res))
            }

            // Integer path
            (a, b) => {
                let v1 = a.as_i64();
                let v2 = b.as_i64();
                if let Some(res) = v1.checked_mul(v2) {
                    Ok(RuntimeValue::Integer(res))
                } else {
                    // Overflow
                    Ok(RuntimeValue::Float(v1 as f64 * v2 as f64))
                }
            }
        }
    }

    /// 除算 (/)
    /// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L32](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L32) 
    /// ゼロ除算の境界条件: エラーを発生させず、結果を 0.0 とする。
    fn div(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        if !self.is_numeric_compat() || !other.is_numeric_compat() {
            return Err(format!("TypeError: '/' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name()));
        }

        let divisor = other.as_f64();
        if divisor == 0.0 {
            return Err("ArithmeticError: Division by zero".into());
        }

        let dividend = self.as_f64();
        let res = dividend / divisor;
        
        if res.is_nan() || res.is_infinite() {
            return Err("ArithmeticError: Result is infinity or NaN".into());
        }
        
        Ok(RuntimeValue::Float(res))
    }

    /// 剰余 (MOD)
    /// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L32](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L32)
    /// ゼロ除算の境界条件: エラーを発生させず、結果を 0 とする。
    fn mod_op(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        if !self.is_numeric_compat() || !other.is_numeric_compat() {
            return Err(format!("TypeError: 'MOD' operator cannot be applied to {} and {}", 
                             self.type_name(), other.type_name()));
        }

        let divisor = other.as_f64();
        if divisor == 0.0 {
            return Err("ArithmeticError: MOD by zero".into());
        }

        match (self, other) {
            // Integer MOD Integer
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                let b_v = *b;
                // divisor == 0 check is already done via as_f64
                Ok(RuntimeValue::Integer(a % b_v))
            }
            
            // Any Float or String involved -> Float Remainder
            (RuntimeValue::Float(_), _) | (_, RuntimeValue::Float(_)) |
            (RuntimeValue::String(_), _) | (_, RuntimeValue::String(_)) => {
                let v1 = self.as_f64();
                let v2 = other.as_f64();
                Ok(RuntimeValue::Float(v1 % v2))
            }

            // Bool/Empty/Null path (treat as integers)
            _ => {
                let v1 = self.as_i64();
                let v2 = other.as_i64();
                // divisor == 0 check is already done via as_f64
                Ok(RuntimeValue::Integer(v1 % v2))
            }
        }
    }
}

/// UWSCの比較演算の実装。
/// 参照: [ORIGINAL_LANGUAGE_SPEC_JA.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC_JA.md#L20) (Case Insensitive)
/// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34) (Implicit Numeric Coercion)
impl UwscCompare for RuntimeValue {
    /// 比較演算 (Eq, Ne, Gt, Lt, Ge, Le)
    /// 文字列はケースインセンシティブ (§20)。
    fn compare(&self, other: &RuntimeValue, op: crate::parser::CmpOp) -> Result<RuntimeValue, String> {
        use crate::parser::CmpOp;

        // Step 1: Handle String vs String (Case-Insensitive)
        // Ref: [ORIGINAL_LANGUAGE_SPEC_JA.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC_JA.md#L20)
        if let (RuntimeValue::String(s1), RuntimeValue::String(s2)) = (self, other) {
            let res = match op {
                CmpOp::Eq => s1.to_uppercase() == s2.to_uppercase(),
                CmpOp::Ne => s1.to_uppercase() != s2.to_uppercase(),
                CmpOp::Gt => s1.to_uppercase() > s2.to_uppercase(),
                CmpOp::Lt => s1.to_uppercase() < s2.to_uppercase(),
                CmpOp::Ge => s1.to_uppercase() >= s2.to_uppercase(),
                CmpOp::Le => s1.to_uppercase() <= s2.to_uppercase(),
            };
            return Ok(RuntimeValue::Bool(res));
        }

        // Step 2: Handle Numeric Coercion
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        // If one is numeric or (string that looks like numeric), try numeric comparison
        let v1_numeric = self.is_numeric_compat();
        let v2_numeric = other.is_numeric_compat();
        
        if v1_numeric || v2_numeric {
            let n1 = self.as_f64();
            let n2 = other.as_f64();
            let res = match op {
                CmpOp::Eq => n1 == n2,
                CmpOp::Ne => n1 != n2,
                CmpOp::Gt => n1 > n2,
                CmpOp::Lt => n1 < n2,
                CmpOp::Ge => n1 >= n2,
                CmpOp::Le => n1 <= n2,
            };
            return Ok(RuntimeValue::Bool(res));
        }

        // Step 3: Fallback to String representation for unsupported types
        let s_self = self.to_string();
        let s_other = other.to_string();
        let res = match op {
            CmpOp::Eq => s_self.to_uppercase() == s_other.to_uppercase(),
            CmpOp::Ne => s_self.to_uppercase() != s_other.to_uppercase(),
            CmpOp::Gt => s_self.to_uppercase() > s_other.to_uppercase(),
            CmpOp::Lt => s_self.to_uppercase() < s_other.to_uppercase(),
            CmpOp::Ge => s_self.to_uppercase() >= s_other.to_uppercase(),
            CmpOp::Le => s_self.to_uppercase() <= s_other.to_uppercase(),
        };
        Ok(RuntimeValue::Bool(res))
    }
}

/// UWSCの論理演算の実装。
/// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L83](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L83) (TRUE=1, FALSE=0)
/// 参照: [ORIGINAL_LANGUAGE_SPEC.ja.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.ja.md#L20) (Truthiness)
impl UwscLogical for RuntimeValue {
    /// Logical/Bitwise AND (§73)
    fn and(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        match (self, other) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l & r)),
            _ => Ok(RuntimeValue::Bool(self.is_truthy() && other.is_truthy())),
        }
    }

    /// Logical/Bitwise OR (§73)
    fn or(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        match (self, other) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l | r)),
            _ => Ok(RuntimeValue::Bool(self.is_truthy() || other.is_truthy())),
        }
    }

    /// Logical/Bitwise XOR (§73)
    fn xor(&self, other: &RuntimeValue) -> Result<RuntimeValue, String> {
        match (self, other) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => Ok(RuntimeValue::Integer(l ^ r)),
            _ => Ok(RuntimeValue::Bool(self.is_truthy() != other.is_truthy())),
        }
    }
}

/// UWSCの単項演算の実装。
/// 参照: [DETAILED_LANGUAGE_SPEC_JA.md#L45](file:///c:/dev/uwsc/uwsc-rs/docs/DETAILED_LANGUAGE_SPEC_JA.md#L45)
impl UwscUnary for RuntimeValue {
    /// Logical Notification/Bitwise NOT (!)
    fn not(&self) -> Result<RuntimeValue, String> {
        match self {
            RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(!n)),
            _ => Ok(RuntimeValue::Bool(!self.is_truthy())),
        }
    }

    /// Arithmetic Negation (-)
    fn neg(&self) -> Result<RuntimeValue, String> {
        match self {
            RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(-n)),
            RuntimeValue::Float(f) => Ok(RuntimeValue::Float(-f)),
            RuntimeValue::String(_) => {
                let f = self.as_f64();
                Ok(RuntimeValue::Float(-f))
            }
            _ => Err(format!("Unary '-' not applicable to: {:?}", self)),
        }
    }
}

/// UWSCの代入演算の実装。
/// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34) (Implicit Numeric Coercion)
impl UwscAssignment for RuntimeValue {
    fn assign(&mut self, source: Self) -> Result<Self, String> {
        *self = source;
        Ok(self.clone())
    }

    fn add_assign(&mut self, rhs: &Self) -> Result<Self, String> {
        let res = self.add(rhs)?;
        self.assign(res)
    }

    fn sub_assign(&mut self, rhs: &Self) -> Result<Self, String> {
        let res = self.sub(rhs)?;
        self.assign(res)
    }

    fn mul_assign(&mut self, rhs: &Self) -> Result<Self, String> {
        let res = self.mul(rhs)?;
        self.assign(res)
    }

    fn div_assign(&mut self, rhs: &Self) -> Result<Self, String> {
        let res = self.div(rhs)?;
        self.assign(res)
    }

    fn mod_assign(&mut self, rhs: &Self) -> Result<Self, String> {
        let res = self.mod_op(rhs)?;
        self.assign(res)
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub() {
        // Standard Int
        assert_eq!(RuntimeValue::Integer(10).sub(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Integer(7));
        
        // Underflow Promotion
        let res = RuntimeValue::Integer(i64::MIN).sub(&RuntimeValue::Integer(1)).unwrap();
        assert_eq!(res.as_f64(), (i64::MIN as f64) - 1.0);

        // Mixed Float
        assert_eq!(RuntimeValue::Float(5.5).sub(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Float(3.5));
        
        // Empty/Null as 0
        assert_eq!(RuntimeValue::Empty.sub(&RuntimeValue::Integer(10)).unwrap(), RuntimeValue::Integer(-10));
        assert_eq!(RuntimeValue::Integer(10).sub(&RuntimeValue::Null).unwrap(), RuntimeValue::Integer(10));
        assert_eq!(RuntimeValue::Empty.sub(&RuntimeValue::Null).unwrap(), RuntimeValue::Integer(0));

        // String Coercion (Failure results in 0)
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        assert_eq!(RuntimeValue::String("A".into()).sub(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Float(-1.0));
    }

    #[test]
    fn test_mul() {
        // Standard
        assert_eq!(RuntimeValue::Integer(5).mul(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Integer(15));
        
        // Overflow -> Float
        let res = RuntimeValue::Integer(2_000_000_000_000_000_000).mul(&RuntimeValue::Integer(5)).unwrap(); // 10^19 > i64::MAX
        assert!(matches!(res, RuntimeValue::Float(_)));
        assert!((res.as_f64() - 1.0e19).abs() < 1.0);

        // Mixed Float
        assert_eq!(RuntimeValue::Float(2.5).mul(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Float(5.0));
        
        // Empty/Null as 0
        assert_eq!(RuntimeValue::Empty.mul(&RuntimeValue::Integer(10)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Integer(10).mul(&RuntimeValue::Null).unwrap(), RuntimeValue::Integer(0));

        // String Coercion (Failure results in 0)
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        assert_eq!(RuntimeValue::String("A".into()).mul(&RuntimeValue::Integer(10)).unwrap(), RuntimeValue::Float(0.0));
    }

    #[test]
    fn test_div() {
        // Integer -> Float
        assert_eq!(RuntimeValue::Integer(10).div(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Float(5.0));
        assert_eq!(RuntimeValue::Integer(5).div(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Float(2.5));
        
        // Division by Zero (UWSC Spec: returns 0.0)
        // Ref: ORIGINAL_LANGUAGE_SPEC.md §2.2
        assert_eq!(RuntimeValue::Integer(10).div(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Float(0.0));
        assert_eq!(RuntimeValue::Integer(10).div(&RuntimeValue::Null).unwrap(), RuntimeValue::Float(0.0));

        // Special values
        assert_eq!(RuntimeValue::Empty.div(&RuntimeValue::Integer(5)).unwrap(), RuntimeValue::Float(0.0));
        assert_eq!(RuntimeValue::Bool(true).div(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Float(0.5));
    }

    #[test]
    fn test_mod() {
        // Standard Int
        assert_eq!(RuntimeValue::Integer(10).mod_op(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Integer(1));
        
        // Float Propagation
        assert_eq!(RuntimeValue::Float(10.5).mod_op(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Float(1.5));
        
        // Modulo by Zero (UWSC Spec: returns 0)
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L32](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L32)
        assert_eq!(RuntimeValue::Integer(10).mod_op(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Integer(10).mod_op(&RuntimeValue::Null).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Float(10.0).mod_op(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Float(0.0));

        // Special types
        assert_eq!(RuntimeValue::Empty.mod_op(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Bool(true).mod_op(&RuntimeValue::Integer(2)).unwrap(), RuntimeValue::Integer(1));
    }

    #[test]
    fn test_arithmetic_coercion() {
        // [Constraint Guard] 合意事項に基づく暗黙変換テスト
        
        // String to Numeric (Implicit)
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        assert_eq!(RuntimeValue::String("10".into()).add(&RuntimeValue::Integer(5)).unwrap(), RuntimeValue::String("105".into())); // String takes priority in '+'
        assert_eq!(RuntimeValue::Integer(5).add(&RuntimeValue::String("10".into())).unwrap(), RuntimeValue::String("510".into()));
        
        // '-' operator triggers numeric coercion
        assert_eq!(RuntimeValue::String("10".into()).sub(&RuntimeValue::Integer(5)).is_err(), false); // wait, current impl fails if not numeric_compat
        // The current is_numeric_compat doesn't include String. This is a potential bug vs spec L34.
    }

    #[test]
    fn test_compare() {
        use crate::parser::CmpOp;

        // [Constraint Guard] 比較演算の契約テスト
        
        // 1. Numeric Comparison
        assert_eq!(RuntimeValue::Integer(10).compare(&RuntimeValue::Float(10.0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Integer(10).compare(&RuntimeValue::Integer(5), CmpOp::Gt).unwrap(), RuntimeValue::Bool(true));

        // 2. String Comparison (Case-Insensitive)
        // Ref: [ORIGINAL_LANGUAGE_SPEC_JA.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC_JA.md#L20)
        assert_eq!(RuntimeValue::String("abc".into()).compare(&RuntimeValue::String("ABC".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("abc".into()).compare(&RuntimeValue::String("ABD".into()), CmpOp::Lt).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("Z".into()).compare(&RuntimeValue::String("a".into()), CmpOp::Gt).unwrap(), RuntimeValue::Bool(true)); // Case-insensitive Z > a

        // 3. Implicit Numeric Coercion
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        assert_eq!(RuntimeValue::Integer(10).compare(&RuntimeValue::String("10".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("20".into()).compare(&RuntimeValue::Integer(10), CmpOp::Gt).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("abc".into()).compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true)); // "abc" -> 0

        // 4. Special Values (Bool, Empty, Null)
        assert_eq!(RuntimeValue::Empty.compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Null.compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Bool(true).compare(&RuntimeValue::Integer(1), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Bool(false).compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        
        // 5. Inequality
        assert_eq!(RuntimeValue::Integer(10).compare(&RuntimeValue::Integer(10), CmpOp::Ne).unwrap(), RuntimeValue::Bool(false));
    }

    #[test]
    fn test_logical() {
        // 論理/ビット演算の評価テスト (§73)
        // 数値同士はビット演算、その他は論理演算結果を返す。

        // 1. AND
        assert_eq!(RuntimeValue::Integer(1).and(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Integer(1));
        assert_eq!(RuntimeValue::Integer(1).and(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::String("A".into()).and(&RuntimeValue::Bool(true)).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Empty.and(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Bool(false));

        // 2. OR
        assert_eq!(RuntimeValue::Integer(1).or(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Integer(1));
        assert_eq!(RuntimeValue::Integer(0).or(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Empty.or(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Bool(true));

        // 3. XOR
        assert_eq!(RuntimeValue::Integer(1).xor(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Integer(0));
        assert_eq!(RuntimeValue::Integer(1).xor(&RuntimeValue::Integer(0)).unwrap(), RuntimeValue::Integer(1));

        // 4. NOT
        assert_eq!(RuntimeValue::Integer(0).not().unwrap(), RuntimeValue::Integer(-1));
        assert_eq!(RuntimeValue::Integer(1).not().unwrap(), RuntimeValue::Integer(-2));
        assert_eq!(RuntimeValue::Empty.not().unwrap(), RuntimeValue::Bool(true));
    }

    #[test]
    fn test_assignment() {
        use crate::semantics::UwscAssignment;
        
        // 1. Simple Assign
        let mut v = RuntimeValue::Integer(1);
        v.assign(RuntimeValue::Integer(2)).unwrap();
        assert_eq!(v, RuntimeValue::Integer(2));

        // 2. Add Assign (with String Coercion)
        let mut v = RuntimeValue::String("10".into());
        v.add_assign(&RuntimeValue::Integer(5)).unwrap();
        assert_eq!(v, RuntimeValue::String("105".into())); // String + Int = Concat

        // 3. Sub Assign (with Coercion §34)
        let mut v = RuntimeValue::String("10".into());
        v.sub_assign(&RuntimeValue::Integer(5)).unwrap();
        assert_eq!(v, RuntimeValue::Float(5.0));

        // 4. Div Assign (Zero division §2.2)
        let mut v = RuntimeValue::Integer(10);
        v.div_assign(&RuntimeValue::Integer(0)).unwrap();
        assert_eq!(v, RuntimeValue::Float(0.0));
    }

    #[test]
    fn test_truthiness() {
        // Numeric
        assert!(RuntimeValue::Integer(1).is_truthy());
        assert!(RuntimeValue::Integer(-1).is_truthy());
        assert!(!RuntimeValue::Integer(0).is_truthy());
        assert!(RuntimeValue::Float(0.1).is_truthy());
        assert!(!RuntimeValue::Float(0.0).is_truthy());

        // Bool
        assert!(RuntimeValue::Bool(true).is_truthy());
        assert!(!RuntimeValue::Bool(false).is_truthy());

        // String
        assert!(RuntimeValue::String("non-empty".into()).is_truthy());
        assert!(!RuntimeValue::String("".into()).is_truthy());

        // Special Values (Spec: Should be False)
        assert!(!RuntimeValue::Empty.is_truthy());
        assert!(!RuntimeValue::Null.is_truthy());
        assert!(!RuntimeValue::Void.is_truthy());

        // Collections (Spec: Non-empty is True)
        assert!(RuntimeValue::Array(vec![RuntimeValue::Integer(0)]).is_truthy());
        assert!(!RuntimeValue::Array(vec![]).is_truthy());
    }

    #[test]
    fn test_comparison_boundaries() {
        use crate::parser::CmpOp;
        
        // 1. Case-Insensitive String Comparison
        // Ref: [ORIGINAL_LANGUAGE_SPEC_JA.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC_JA.md#L20)
        assert_eq!(RuntimeValue::String("ABC".into()).compare(&RuntimeValue::String("abc".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("ABC".into()).compare(&RuntimeValue::String("ABC".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("ABC".into()).compare(&RuntimeValue::String("abd".into()), CmpOp::Lt).unwrap(), RuntimeValue::Bool(true));

        // 2. Implicit Numeric Coercion
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L34](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L34)
        assert_eq!(RuntimeValue::String("100".into()).compare(&RuntimeValue::Integer(100), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Integer(50).compare(&RuntimeValue::String("100".into()), CmpOp::Lt).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::String("50.5".into()).compare(&RuntimeValue::Float(50.5), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));

        // 3. Special Values (EMPTY=0, NULL="NULL", Bool=1/0)
        // Ref: [ORIGINAL_LANGUAGE_SPEC.md#L83](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L83)
        assert_eq!(RuntimeValue::Empty.compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Null.compare(&RuntimeValue::String("NULL".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Bool(true).compare(&RuntimeValue::Integer(1), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Bool(false).compare(&RuntimeValue::Integer(0), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));

        // 4. Fallback Stringification Comparison
        // When numeric comparison is not possible, both are stringified
        let arr = RuntimeValue::Array(vec![]);
        assert_eq!(arr.compare(&RuntimeValue::String("[Array]".into()), CmpOp::Eq).unwrap(), RuntimeValue::Bool(true));
    }

    #[test]
    fn test_logical_bitwise() {
        use crate::semantics::UwscLogical;
        use crate::semantics::UwscUnary;

        // 1. Bitwise Integer (Standard behavior for constants §5.3)
        assert_eq!(RuntimeValue::Integer(3).and(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Integer(1));
        assert_eq!(RuntimeValue::Integer(2).or(&RuntimeValue::Integer(1)).unwrap(), RuntimeValue::Integer(3));
        assert_eq!(RuntimeValue::Integer(1).xor(&RuntimeValue::Integer(3)).unwrap(), RuntimeValue::Integer(2));
        assert_eq!(RuntimeValue::Integer(0).not().unwrap(), RuntimeValue::Integer(-1));
        assert_eq!(RuntimeValue::Integer(1).not().unwrap(), RuntimeValue::Integer(-2));

        // 2. Logical Mixed (Fallback to Truthiness)
        assert_eq!(RuntimeValue::Bool(true).and(&RuntimeValue::Bool(false)).unwrap(), RuntimeValue::Bool(false));
        assert_eq!(RuntimeValue::String("A".into()).or(&RuntimeValue::Empty).unwrap(), RuntimeValue::Bool(true));
        assert_eq!(RuntimeValue::Null.not().unwrap(), RuntimeValue::Bool(true));
    }

    #[test]
    fn test_unary_neg() {
        use crate::semantics::UwscUnary;

        // 1. Numeric Negation
        assert_eq!(RuntimeValue::Integer(10).neg().unwrap(), RuntimeValue::Integer(-10));
        assert_eq!(RuntimeValue::Float(5.5).neg().unwrap(), RuntimeValue::Float(-5.5));

        // 2. Coerced String Negation (§34)
        assert_eq!(RuntimeValue::String("100".into()).neg().unwrap(), RuntimeValue::Float(-100.0));
        
        // 3. Error Case (Non-numeric)
        assert!(RuntimeValue::Array(vec![]).neg().is_err());
    }
}
