use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct StringBuiltins;

impl BuiltinModule for StringBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("LENGTH".into(), length);
        r.insert("COPY".into(), copy);
        r.insert("POS".into(), pos);
        r.insert("REPLACE".into(), replace);
        r.insert("CHGMOJ".into(), replace);
        r.insert("TRIM".into(), trim);
        r.insert("VAL".into(), val);
        r.insert("FORMAT".into(), format);
        r.insert("TOKEN".into(), token);
        r.insert("SPLIT".into(), split);
        r.insert("JOIN".into(), join);
        r.insert("BETWEENSTR".into(), betweenstr);
    }
}

fn length(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("LENGTH requires 1 argument".into()); }
    match &args[0] {
        RuntimeValue::Array(a) => Ok(RuntimeValue::Integer(a.len() as i64)),
        _ => Ok(RuntimeValue::Integer(args[0].to_string().chars().count() as i64)),
    }
}

fn copy(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("COPY requires at least 2 arguments".into()); }
    let s = args[0].to_string();
    let start = args[1].as_i64();
    if start < 1 { return Ok(RuntimeValue::String("".into())); }
    let chars: Vec<char> = s.chars().collect();
    let start_idx = (start - 1) as usize;
    if start_idx >= chars.len() { return Ok(RuntimeValue::String("".into())); }
    
    let length_val = if args.len() > 2 { Some(args[2].as_i64()) } else { None };
    if let Some(l) = length_val {
        if l <= 0 { return Ok(RuntimeValue::String("".into())); }
        let l_usize = l as usize;
        let result = chars.iter().skip(start_idx).take(l_usize).collect::<String>();
        Ok(RuntimeValue::String(result))
    } else {
        let result = chars.iter().skip(start_idx).collect::<String>();
        Ok(RuntimeValue::String(result))
    }
}

fn pos(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("POS requires at least 2 arguments".into()); }
    let substr = args[0].to_string();
    let s = args[1].to_string();
    let n = if args.len() > 2 { args[2].as_i64() } else { 1 };
    if substr.is_empty() || n == 0 { return Ok(RuntimeValue::Integer(0)); }
    let chars: Vec<char> = s.chars().collect();
    let sub_chars: Vec<char> = substr.chars().collect();
    let mut matches = Vec::new();
    for i in 0..=(chars.len().saturating_sub(sub_chars.len())) {
        if &chars[i..i+sub_chars.len()] == sub_chars.as_slice() {
            matches.push(i + 1);
        }
    }
    let result = if n > 0 {
        matches.get((n - 1) as usize).cloned().unwrap_or(0)
    } else {
        let idx = (matches.len() as i64 + n) as usize;
        matches.get(idx).cloned().unwrap_or(0)
    };
    Ok(RuntimeValue::Integer(result as i64))
}

fn replace(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("REPLACE requires 3 arguments".into()); }
    let s = args[0].to_string();
    let old = args[1].to_string();
    let new = args[2].to_string();
    if old.is_empty() { return Ok(RuntimeValue::String(s)); }
    Ok(RuntimeValue::String(s.replace(&old, &new)))
}

fn trim(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("TRIM requires 1 argument".into()); }
    let s = args[0].to_string();
    let trimmed = s.trim_matches(|c: char| c.is_whitespace() || c == '\u{3000}');
    Ok(RuntimeValue::String(trimmed.to_string()))
}

fn val(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("VAL requires at least 1 argument".into()); }
    let s = args[0].to_string().trim().to_string();
    let default_val = if args.len() > 1 { args[1].clone() } else { RuntimeValue::Integer(0) };
    if s.is_empty() { return Ok(default_val); }
    if let Ok(i) = s.parse::<i64>() { return Ok(RuntimeValue::Integer(i)); }
    if let Ok(f) = s.parse::<f64>() { return Ok(RuntimeValue::Float(f)); }
    Ok(default_val)
}

fn format(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("FORMAT requires at least 1 argument".into()); }
    let val = &args[0];
    let width = if args.len() > 1 { args[1].as_i64() } else { 0 };
    let decimal = if args.len() > 2 { Some(args[2].as_i64() as usize) } else { None };

    let s = match (val, decimal) {
        (RuntimeValue::Float(f), Some(d)) => format!("{:.*}", d, f),
        (RuntimeValue::Integer(i), Some(d)) => format!("{:.*}", d, *i as f64),
        _ => val.to_string(),
    };

    let result = if width > 0 {
        format!("{:>width$}", s, width = width as usize)
    } else if width < 0 {
        format!("{:<width$}", s, width = (-width) as usize)
    } else {
        s
    };

    Ok(RuntimeValue::String(result))
}

fn token(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("TOKEN requires 2 arguments".into()); }
    let sep = args[0].to_string();
    let s = args[1].to_string();
    
    if s.is_empty() {
        b.last_token_remaining = Some("".into());
        return Ok(RuntimeValue::String("".into()));
    }

    let sep_chars: Vec<char> = sep.chars().collect();
    let s_chars: Vec<char> = s.chars().collect();
    
    let mut split_idx = None;
    for (i, &c) in s_chars.iter().enumerate() {
        if sep_chars.contains(&c) {
            split_idx = Some(i);
            break;
        }
    }

    match split_idx {
        Some(idx) => {
            let token = s_chars[..idx].iter().collect::<String>();
            let rest = s_chars[idx+1..].iter().collect::<String>();
            b.last_token_remaining = Some(rest);
            Ok(RuntimeValue::String(token))
        }
        None => {
            b.last_token_remaining = Some("".into());
            Ok(RuntimeValue::String(s))
        }
    }
}

fn split(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("SPLIT requires at least 1 argument".into()); }
    let s = args[0].to_string();
    let sep = if args.len() > 1 { args[1].to_string() } else { ",".into() };
    
    if sep.is_empty() {
        let items = s.chars().map(|c| RuntimeValue::String(c.to_string())).collect();
        return Ok(RuntimeValue::Array(items));
    }

    let items = s.split(&sep).map(|part| RuntimeValue::String(part.to_string())).collect();
    Ok(RuntimeValue::Array(items))
}

fn join(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("JOIN requires at least 1 argument".into()); }
    let arr = match &args[0] {
        RuntimeValue::Array(a) => a,
        _ => return Err("JOIN requires an array as the first argument".into()),
    };
    let sep = if args.len() > 1 { args[1].to_string() } else { "".into() };
    
    let result = arr.iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(&sep);
        
    Ok(RuntimeValue::String(result))
}

fn betweenstr(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("BETWEENSTR requires 3 arguments".into()); }
    let s = args[0].to_string();
    let pre = args[1].to_string();
    let post = args[2].to_string();
    let n = args.get(3).map(|v| v.as_i64() as usize).unwrap_or(1);

    if pre.is_empty() && post.is_empty() { return Ok(RuntimeValue::String(s)); }

    let mut current_pos = 0;
    for i in 0..n {
        if let Some(start_idx) = s[current_pos..].find(&pre) {
            let actual_start = current_pos + start_idx + pre.len();
            if let Some(end_idx) = s[actual_start..].find(&post) {
                if i == n - 1 {
                    return Ok(RuntimeValue::String(s[actual_start..actual_start + end_idx].to_string()));
                }
                current_pos = actual_start + end_idx + post.len();
            } else {
                return Ok(RuntimeValue::String("".into()));
            }
        } else {
            return Ok(RuntimeValue::String("".into()));
        }
    }
    Ok(RuntimeValue::String("".into()))
}
