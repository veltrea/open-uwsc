use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use screenshots::Screen;

pub struct ImageBuiltins;

impl BuiltinModule for ImageBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("CHKIMG".into(), chkimg);
    }
}

fn chkimg(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("CHKIMG requires an image path".into()); }
    let template_path = args[0].to_string();
    let x1 = if args.len() >= 3 { args[2].as_i64() as i32 } else { 0 };
    let y1 = if args.len() >= 4 { args[3].as_i64() as i32 } else { 0 };
    let x2 = if args.len() >= 5 { args[4].as_i64() as i32 } else { -1 };
    let y2 = if args.len() >= 6 { args[5].as_i64() as i32 } else { -1 };
    let tolerance = if args.len() >= 7 { args[6].as_i64() as i32 } else { 0 };

    let screens = Screen::all().map_err(|e| format!("CHKIMG: Failed to list screens: {}", e))?;
    if screens.is_empty() { return Ok(RuntimeValue::Bool(false)); }
    let screen = screens[0];
    let screen_image = screen.capture().map_err(|e| format!("CHKIMG: Screenshot failed: {}", e))?;
    let template = image::open(&template_path).map_err(|e| format!("CHKIMG: Template failed: {}", e))?.to_rgba8();
    
    let (sw, sh) = screen_image.dimensions();
    let (tw, th) = template.dimensions();
    let start_x = x1.max(0) as u32;
    let start_y = y1.max(0) as u32;
    let end_x = if x2 < 0 { sw.saturating_sub(tw) } else { (x2 as u32).min(sw.saturating_sub(tw)) };
    let end_y = if y2 < 0 { sh.saturating_sub(th) } else { (y2 as u32).min(sh.saturating_sub(th)) };

    if tw > sw || th > sh || start_x > end_x || start_y > end_y { return Ok(RuntimeValue::Bool(false)); }

    let first_pixel = template.get_pixel(0, 0);
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            if !pixel_match(screen_image.get_pixel(x, y), first_pixel, tolerance) { continue; }
            let mut matched = true;
            'inner: for ty in 0..th {
                for tx in 0..tw {
                    if !pixel_match(screen_image.get_pixel(x + tx, y + ty), template.get_pixel(tx, ty), tolerance) {
                        matched = false;
                        break 'inner;
                    }
                }
            }
            if matched {
                b.last_img_x = x as i32;
                b.last_img_y = y as i32;
                b.last_img_id = 0;
                return Ok(RuntimeValue::Bool(true));
            }
        }
    }
    Ok(RuntimeValue::Bool(false))
}

fn pixel_match(p1: &image::Rgba<u8>, p2: &image::Rgba<u8>, tolerance: i32) -> bool {
    if tolerance == 0 { return p1 == p2; }
    for i in 0..3 {
        let diff = (p1[i] as i32 - p2[i] as i32).abs();
        if diff > tolerance { return false; }
    }
    true
}
