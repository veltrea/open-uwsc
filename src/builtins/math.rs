use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct MathBuiltins;

impl BuiltinModule for MathBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("RANDOM".into(), random);
        r.insert("ABS".into(), abs);
        r.insert("INT".into(), int);
        r.insert("ROUND".into(), round);
        r.insert("SQRT".into(), sqrt);
    }
}

fn sqrt(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("SQRT requires 1 argument".into()); }
    let val = args[0].as_f64();
    if val < 0.0 { return Err("SQRT of negative number".into()); }
    Ok(RuntimeValue::Float(val.sqrt()))
}


fn random(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("RANDOM requires 1 argument".into()); }
    let max = args[0].as_i64();
    if max <= 0 { return Ok(RuntimeValue::Integer(0)); }
    use rand::RngExt;
    let mut rng = rand::rng();
    Ok(RuntimeValue::Integer(rng.random_range(0..max)))
}

fn abs(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("ABS requires 1 argument".into()); }
    match args[0] {
        RuntimeValue::Integer(n) => Ok(RuntimeValue::Integer(n.abs())),
        RuntimeValue::Float(f) => Ok(RuntimeValue::Float(f.abs())),
        _ => Ok(RuntimeValue::Float(args[0].as_f64().abs())),
    }
}

fn int(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("INT requires 1 argument".into()); }
    Ok(RuntimeValue::Integer(args[0].as_i64()))
}

fn round(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("ROUND requires 1 argument".into()); }
    let f = args[0].as_f64();
    Ok(RuntimeValue::Integer(f.round() as i64))
}
