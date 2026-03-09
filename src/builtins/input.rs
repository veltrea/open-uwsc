use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct InputBuiltins;

impl BuiltinModule for InputBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("MMV".into(), mmv);
        r.insert("BTN".into(), btn);
        r.insert("KBD".into(), kbd);
    }
}

fn mmv(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("MMV requires 2 arguments (x, y)".into()); }
    let x = args[0].as_i64();
    let y = args[1].as_i64();
    let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    unsafe {
        let mut input = INPUT::default();
        input.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(0); // INPUT_MOUSE
        input.Anonymous.mi = MOUSEINPUT {
            dx: (x * 65535 / w as i64) as i32,
            dy: (y * 65535 / h as i64) as i32,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
    Ok(RuntimeValue::Bool(true))
}

fn btn(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("BTN requires 2 arguments (button, state)".into()); }
    let button = args[0].as_i64();
    let state = args[1].as_i64();
    if args.len() >= 4 {
        let x = args[2].as_i64();
        let y = args[3].as_i64();
        let _ = mmv(b, &mut [RuntimeValue::Integer(x), RuntimeValue::Integer(y)]);
    }
    let (down_flag, up_flag) = match button {
        0 => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        1 => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        2 => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => return Err(format!("BTN: Unknown button {}", button)),
    };
    unsafe {
        if state == 0 || state == 1 {
            let mut input = INPUT::default();
            input.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(0);
            input.Anonymous.mi.dwFlags = down_flag;
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        if state == 0 || state == 2 {
            let mut input = INPUT::default();
            input.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(0);
            input.Anonymous.mi.dwFlags = up_flag;
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }
    Ok(RuntimeValue::Bool(true))
}

fn kbd(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("KBD requires at least 1 argument".into()); }
    let vk_code = match &args[0] {
        RuntimeValue::Integer(n) => *n as u16,
        RuntimeValue::String(s) => {
            if s.len() == 1 { s.chars().next().unwrap().to_ascii_uppercase() as u16 } else { 0 }
        }
        _ => 0,
    };
    if vk_code == 0 { return Ok(RuntimeValue::Empty); }
    let k_type = if args.len() > 1 { args[1].as_i64() } else { 0 };
    unsafe {
        let mut inputs = Vec::new();
        if k_type == 0 || k_type == 1 {
            inputs.push(INPUT {
                r#type: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1), // INPUT_KEYBOARD
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(vk_code),
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        if k_type == 0 || k_type == 2 {
            inputs.push(INPUT {
                r#type: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1),
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(vk_code),
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
    Ok(RuntimeValue::Empty)
}
