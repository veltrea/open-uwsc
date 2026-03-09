use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;

pub struct BrowserBuiltins;

impl BuiltinModule for BrowserBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("BROWSER_OPEN".into(), browser_open);
        r.insert("BROWSER_INPUT".into(), browser_input);
        r.insert("BROWSER_CLICK".into(), browser_click);
        r.insert("BROWSER_CLOSE".into(), browser_close);
    }
}

fn browser_open(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("BROWSER_OPEN requires 1 argument (url)".into()); }
    let url = args[0].to_string();
    let headful = if args.len() > 1 { args[1].as_bool() } else { true };

    let options = LaunchOptions {
        headless: !headful,
        ..Default::default()
    };

    let browser = Browser::new(options).map_err(|e| format!("Failed to launch browser: {}", e))?;
    let tab = browser.new_tab().map_err(|e| format!("Failed to open tab: {}", e))?;
    
    tab.navigate_to(&url).map_err(|e| format!("Failed to navigate: {}", e))?;

    b.browser_tab = Some(tab);
    b.browser = Some(browser);
    
    Ok(RuntimeValue::Bool(true))
}

fn browser_input(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("BROWSER_INPUT requires 2 arguments (selector, text)".into()); }
    let tab = b.browser_tab.as_ref().ok_or("No active browser session")?;
    
    let selector = args[0].to_string();
    let text = args[1].to_string();

    let element = tab.wait_for_element(&selector).map_err(|e| format!("Element not found: {}", e))?;
    element.type_into(&text).map_err(|e| format!("Failed to input text: {}", e))?;

    Ok(RuntimeValue::Bool(true))
}

fn browser_click(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("BROWSER_CLICK requires 1 argument (selector)".into()); }
    let tab = b.browser_tab.as_ref().ok_or("No active browser session")?;
    
    let selector = args[0].to_string();

    let element = tab.wait_for_element(&selector).map_err(|e| format!("Element not found: {}", e))?;
    element.click().map_err(|e| format!("Failed to click: {}", e))?;

    Ok(RuntimeValue::Bool(true))
}

fn browser_close(b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    b.browser_tab = None;
    Ok(RuntimeValue::Bool(true))
}
