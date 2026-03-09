use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use chrono::Local;

pub struct SystemBuiltins;

impl BuiltinModule for SystemBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("PRINT".into(), print);
        r.insert("SLEEP".into(), sleep_func);
        r.insert("GETTIME".into(), gettime);
        r.insert("STDOUT".into(), stdout_func);
        r.insert("STDERR".into(), stderr_func);
        r.insert("STDIN".into(), stdin_func);
        r.insert("CREATEOLEOBJ".into(), createoleobj);
        r.insert("DEF_DLL".into(), def_dll);
        r.insert("GET_WIN_DIR".into(), get_win_dir);
        r.insert("GET_SYS_DIR".into(), get_sys_dir);
        r.insert("GET_CUR_DIR".into(), get_cur_dir);
        r.insert("GET_APPDATA_DIR".into(), get_appdata_dir);
    }
}

fn print(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("PRINT requires arg".into()); }
    println!("{}", args[0].to_string());
    Ok(RuntimeValue::Empty)
}

fn sleep_func(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("SLEEP requires 1 argument".into()); }
    let secs = args[0].as_f64();
    if secs > 0.0 { thread::sleep(Duration::from_secs_f64(secs)); }
    Ok(RuntimeValue::Bool(true))
}

fn gettime(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    use chrono::{Duration as ChronoDuration, Datelike, Timelike};
    let offset = if !args.is_empty() { args[0].as_i64() } else { 0 };
    let now = Local::now() + ChronoDuration::seconds(offset);
    
    b.g_time_yy = now.year();
    b.g_time_mm = now.month() as i32;
    b.g_time_dd = now.day() as i32;
    b.g_time_hh = now.hour() as i32;
    b.g_time_nn = now.minute() as i32;
    b.g_time_ss = now.second() as i32;
    b.g_time_zz = (now.timestamp_subsec_millis()) as i32;
    b.g_time_ww = (now.weekday().num_days_from_sunday()) as i32;

    Ok(RuntimeValue::Integer(now.timestamp()))
}

fn createoleobj(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("CREATEOLEOBJ requires a ProgID".into()); }
    Ok(RuntimeValue::String(format!("OLEObject({})", args[0].to_string())))
}

fn stdout_func(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("STDOUT requires arg".into()); }
    let msg = args[0].to_string();
    let no_newline = if args.len() > 1 { args[1].as_bool() } else { false };
    
    use std::io::{Write, stdout};
    if no_newline {
        print!("{}", msg);
        stdout().flush().ok();
    } else {
        println!("{}", msg);
    }
    Ok(RuntimeValue::Empty)
}

fn stderr_func(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("STDERR requires arg".into()); }
    eprintln!("{}", args[0].to_string());
    Ok(RuntimeValue::Empty)
}

fn stdin_func(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    use std::io::{stdin, BufRead};
    let mut line = String::new();
    let stdin = stdin();
    let mut handle = stdin.lock();
    handle.read_line(&mut line).map_err(|e| format!("STDIN read failed: {}", e))?;
    Ok(RuntimeValue::String(line.trim_end_matches(['\r', '\n']).to_string()))
}

fn def_dll(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::Empty)
}

fn get_win_dir(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::String(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string())))
}

fn get_sys_dir(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    let win = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    Ok(RuntimeValue::String(format!("{}\\System32", win)))
}

fn get_cur_dir(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::String(std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()))
}

fn get_appdata_dir(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::String(std::env::var("APPDATA").unwrap_or_default()))
}
