use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextW, GetClassNameW, GetWindowRect, IsWindowVisible,
    GetWindowThreadProcessId, GetWindowLongW, GWL_STYLE,
};
use windows::Win32::Foundation::{HWND, RECT};

pub struct InspectionBuiltins;

impl BuiltinModule for InspectionBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("GET_WIN_INFO".into(), get_win_info);
        r.insert("GET_UIA_INFO".into(), get_uia_info);
    }
}

fn get_win_info(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("GET_WIN_INFO requires 1 argument (id)".into()); }
    let hwnd_val = args[0].as_i64();
    if hwnd_val <= 0 { return Ok(RuntimeValue::Empty); }
    let hwnd = HWND(hwnd_val as *mut _);

    let mut map = HashMap::new();
    unsafe {
        // Text
        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        map.insert("TITLE".to_uppercase(), RuntimeValue::String(String::from_utf16_lossy(&text[..len as usize])));

        // Class
        let mut class_text = [0u16; 512];
        let class_len = GetClassNameW(hwnd, &mut class_text);
        map.insert("CLASS".to_uppercase(), RuntimeValue::String(String::from_utf16_lossy(&class_text[..class_len as usize])));

        // Rect
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        map.insert("X".to_uppercase(), RuntimeValue::Integer(rect.left as i64));
        map.insert("Y".to_uppercase(), RuntimeValue::Integer(rect.top as i64));
        map.insert("WIDTH".to_uppercase(), RuntimeValue::Integer((rect.right - rect.left) as i64));
        map.insert("HEIGHT".to_uppercase(), RuntimeValue::Integer((rect.bottom - rect.top) as i64));

        // State
        map.insert("VISIBLE".to_uppercase(), RuntimeValue::Bool(IsWindowVisible(hwnd).as_bool()));

        // Process Info
        let mut pid = 0u32;
        let thread_id = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        map.insert("PID".to_uppercase(), RuntimeValue::Integer(pid as i64));
        map.insert("THREAD_ID".to_uppercase(), RuntimeValue::Integer(thread_id as i64));

        // Style
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        map.insert("STYLE".to_uppercase(), RuntimeValue::Integer(style as i64));
    }

    Ok(RuntimeValue::Hash { map, case_care: false })
}

fn get_uia_info(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    // Basic placeholder for UIA integration
    Ok(RuntimeValue::String("UIA info coming soon...".into()))
}
