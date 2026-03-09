use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct ArrayBuiltins;

impl BuiltinModule for ArrayBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("SLICE".into(), slice);
        r.insert("QSORT".into(), qsort);
        r.insert("RESIZE".into(), resize);
        r.insert("SETCLEAR".into(), setclear);
    }
}

fn slice(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("SLICE requires an array".into()); }
    let arr = match &args[0] {
        RuntimeValue::Array(a) => a,
        _ => return Err("SLICE first argument must be an array".into()),
    };
    let start = args.get(1).map(|v| v.as_i64() as usize).unwrap_or(0);
    let end = args.get(2).map(|v| v.as_i64() as usize).unwrap_or(arr.len());

    let start = start.min(arr.len());
    let end = end.min(arr.len()).max(start);

    Ok(RuntimeValue::Array(arr[start..end].to_vec()))
}

fn qsort(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("QSORT requires an array".into()); }
    let mut arr = match &args[0] {
        RuntimeValue::Array(a) => a.clone(),
        _ => return Err("QSORT argument must be an array".into()),
    };

    // Simple sort for now (ascending)
    arr.sort_by(|a, b| {
        let sa = a.to_string();
        let sb = b.to_string();
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    });

    b.last_modified_array = Some(arr);
    Ok(RuntimeValue::Empty)
}

fn resize(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("RESIZE requires an array and size".into()); }
    let mut arr = match &args[0] {
        RuntimeValue::Array(a) => a.clone(),
        _ => return Err("RESIZE first argument must be an array".into()),
    };
    let new_size = args[1].as_i64() as usize;

    if new_size > arr.len() {
        for _ in 0..(new_size - arr.len()) {
            arr.push(RuntimeValue::Empty);
        }
    } else {
        arr.truncate(new_size);
    }

    b.last_modified_array = Some(arr);
    Ok(RuntimeValue::Empty)
}

fn setclear(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("SETCLEAR requires an array".into()); }
    let mut arr = match &args[0] {
        RuntimeValue::Array(a) => a.clone(),
        _ => return Err("SETCLEAR first argument must be an array".into()),
    };
    let val = args.get(1).cloned().unwrap_or(RuntimeValue::Empty);

    for e in arr.iter_mut() {
        *e = val.clone();
    }

    b.last_modified_array = Some(arr);
    Ok(RuntimeValue::Empty)
}
