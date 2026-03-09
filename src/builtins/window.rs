use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use crate::semantics::UwscArithmetic; // Fixed private import
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_OK, IDOK, IDCANCEL, IDYES, IDNO,
    ShowWindow, SetForegroundWindow, PostMessageW, SendMessageW,
    GetWindowRect, GetWindowTextW, GetClassNameW, IsWindowVisible,
    GetForegroundWindow, SetWindowPos, IsZoomed,
    SW_HIDE, SW_SHOWNORMAL, SW_SHOWMINIMIZED, SW_SHOWMAXIMIZED, SW_SHOW,
    WM_CLOSE, WM_SETTEXT,
};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM, RECT};
use windows::core::HSTRING;

pub struct WindowBuiltins;

impl BuiltinModule for WindowBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("MSGBOX".into(), msgbox);
        r.insert("GETID".into(), getid);
        r.insert("CTRLWIN".into(), ctrlwin);
        r.insert("ACW".into(), acw);
        r.insert("STATUS".into(), status);
        r.insert("SENDSTR".into(), sendstr);
    }
}

fn msgbox(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("MSGBOX requires at least 1 argument".into()); }
    
    let msg = HSTRING::from(args[0].to_string());
    let btns = if args.len() > 1 {
        match args[1] {
            RuntimeValue::Integer(n) => windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE(n as u32),
            _ => MB_OK,
        }
    } else {
        MB_OK
    };

    unsafe {
        let result = MessageBoxW(None, &msg, &HSTRING::from("UWSC-RS"), btns);
        let ret_val = match result {
            IDOK => 1,
            IDCANCEL => 2,
            IDYES => 6,
            IDNO => 7,
            _ => result.0 as i64,
        };
        Ok(RuntimeValue::Integer(ret_val))
    }
}

fn getid(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    let title = if !args.is_empty() { args[0].to_string() } else { "".into() };
    let class = if args.len() > 1 { args[1].to_string() } else { "".into() };

    if title == "GET_ACTIVE_WIN" {
        let hwnd = unsafe { GetForegroundWindow() };
        return Ok(RuntimeValue::Integer(hwnd.0 as i64));
    }

    let mut result_hwnd = HWND(std::ptr::null_mut());
    let title_up = title.to_uppercase();
    let class_up = class.to_uppercase();

    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(enum_windows_callback), 
            LPARAM(&mut (&title_up, &class_up, &mut result_hwnd) as *mut _ as isize)
        );
    }

    if result_hwnd.0.is_null() {
        Ok(RuntimeValue::Integer(-1))
    } else {
        Ok(RuntimeValue::Integer(result_hwnd.0 as i64))
    }
}

extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    unsafe {
        let ptr = lparam.0 as *mut (&String, &String, &mut HWND);
        let (target_title, target_class, out_hwnd) = &mut *ptr;

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        let title = String::from_utf16_lossy(&text[..len as usize]).to_uppercase();

        let mut class_text = [0u16; 512];
        let class_len = GetClassNameW(hwnd, &mut class_text);
        let class = String::from_utf16_lossy(&class_text[..class_len as usize]).to_uppercase();

        let title_match = target_title.is_empty() || title.contains(target_title.as_str());
        let class_match = target_class.is_empty() || class.contains(target_class.as_str());

        if title_match && class_match {
            **out_hwnd = hwnd;
            return false.into(); 
        }

        true.into()
    }
}

fn ctrlwin(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("CTRLWIN requires 2 arguments (id, action)".into()); }
    
    let hwnd_val = match args[0] {
        RuntimeValue::Integer(n) => n,
        _ => return Err("CTRLWIN: id must be an integer".into()),
    };
    if hwnd_val <= 0 { return Ok(RuntimeValue::Bool(false)); }
    
    let hwnd = HWND(hwnd_val as *mut _);
    let action = match args[1] {
        RuntimeValue::Integer(n) => n,
        _ => return Err("CTRLWIN: action must be an integer".into()),
    };

    unsafe {
        match action {
            1 => { let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)); }
            2 => { let _ = ShowWindow(hwnd, SW_SHOW); let _ = SetForegroundWindow(hwnd); }
            3 => { let _ = ShowWindow(hwnd, SW_SHOWMINIMIZED); }
            4 => { let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED); }
            5 => { let _ = ShowWindow(hwnd, SW_SHOWNORMAL); }
            6 => { let _ = ShowWindow(hwnd, SW_HIDE); }
            7 => { let _ = ShowWindow(hwnd, SW_SHOW); }
            _ => return Err(format!("CTRLWIN: Unknown action {}", action)),
        }
    }
    Ok(RuntimeValue::Bool(true))
}

fn acw(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("ACW requires at least 3 arguments (id, x, y)".into()); }
    let hwnd_val = args[0].as_i64();
    if hwnd_val <= 0 { return Ok(RuntimeValue::Bool(false)); }
    let hwnd = HWND(hwnd_val as *mut _);
    let x = args[1].as_i64() as i32;
    let y = args[2].as_i64() as i32;
    let mut width = -1;
    let mut height = -1;
    let mut flags = 0;
    if args.len() > 3 { width = args[3].as_i64() as i32; }
    if args.len() > 4 { height = args[4].as_i64() as i32; }
    if width < 0 && height < 0 {
        flags |= 0x0001; // SWP_NOSIZE
    } else if width < 0 || height < 0 {
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        unsafe { let _ = GetWindowRect(hwnd, &mut rect); }
        if width < 0 { width = rect.right - rect.left; }
        if height < 0 { height = rect.bottom - rect.top; }
    }
    unsafe {
        let _ = SetWindowPos(hwnd, None, x, y, width, height, windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS(0x0004 | 0x0040 | flags as u32));
    }
    Ok(RuntimeValue::Bool(true))
}

fn status(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("STATUS requires 2 arguments (id, type)".into()); }
    let hwnd_val = args[0].as_i64();
    if hwnd_val <= 0 { return Ok(RuntimeValue::Empty); }
    let hwnd = HWND(hwnd_val as *mut _);
    let st_type = args[1].as_i64();
    unsafe {
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let _ = GetWindowRect(hwnd, &mut rect);
        match st_type {
            0 => { 
                let mut text = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut text);
                Ok(RuntimeValue::String(String::from_utf16_lossy(&text[..len as usize])))
            }
            1 => Ok(RuntimeValue::Integer(rect.left as i64)),
            2 => Ok(RuntimeValue::Integer(rect.top as i64)),
            3 => Ok(RuntimeValue::Integer((rect.right - rect.left) as i64)),
            4 => Ok(RuntimeValue::Integer((rect.bottom - rect.top) as i64)),
            8 => Ok(RuntimeValue::Bool(IsZoomed(hwnd).as_bool())),
            9 => Ok(RuntimeValue::Bool(IsWindowVisible(hwnd).as_bool())),
            10 => {
                let active = GetForegroundWindow();
                Ok(RuntimeValue::Bool(active == hwnd))
            }
            _ => Ok(RuntimeValue::Empty),
        }
    }
}

fn sendstr(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("SENDSTR requires 2 arguments (id, str)".into()); }
    let hwnd_val = match args[0] {
        RuntimeValue::Integer(n) => n,
        _ => return Err("SENDSTR: id must be an integer".into()),
    };
    if hwnd_val <= 0 { return Ok(RuntimeValue::Bool(false)); }
    let hwnd = HWND(hwnd_val as *mut _);
    let text = args[1].to_string();
    let hstring = HSTRING::from(&text);
    unsafe {
        let _ = SendMessageW(hwnd, WM_SETTEXT, Some(WPARAM(0)), Some(LPARAM(hstring.as_ptr() as isize)));
    }
    Ok(RuntimeValue::Bool(true))
}
