use crate::environment::Environment;
use crate::value::RuntimeValue;

pub fn init(g: &mut Environment) {
    // Window Constants (§8 / §9)
    g.define("CLOSE".into(), RuntimeValue::Integer(1), true);
    g.define("ACTIVATE".into(), RuntimeValue::Integer(2), true);
    g.define("MIN".into(), RuntimeValue::Integer(3), true);
    g.define("MAX".into(), RuntimeValue::Integer(4), true);
    g.define("NORMAL".into(), RuntimeValue::Integer(5), true);
    g.define("HIDE".into(), RuntimeValue::Integer(6), true);
    g.define("SHOW".into(), RuntimeValue::Integer(7), true);
    
    // Mouse/Keyboard Constants (§13 / §14)
    g.define("LEFT".into(), RuntimeValue::Integer(0), true);
    g.define("RIGHT".into(), RuntimeValue::Integer(1), true);
    g.define("MIDDLE".into(), RuntimeValue::Integer(2), true);
    g.define("CLICK".into(), RuntimeValue::Integer(0), true);
    g.define("DOWN".into(), RuntimeValue::Integer(1), true);
    g.define("UP".into(), RuntimeValue::Integer(2), true);
    
    // Virtual Keys
    g.define("VK_RETURN".into(), RuntimeValue::Integer(0x0D), true);
    g.define("VK_ENTER".into(), RuntimeValue::Integer(0x0D), true);
    g.define("VK_SPACE".into(), RuntimeValue::Integer(0x20), true);
    g.define("VK_BACK".into(), RuntimeValue::Integer(0x08), true);
    g.define("VK_TAB".into(), RuntimeValue::Integer(0x09), true);
    g.define("VK_ESCAPE".into(), RuntimeValue::Integer(0x1B), true);
    
    // STATUS Constants
    g.define("ST_TITLE".into(), RuntimeValue::Integer(0), true);
    g.define("ST_X".into(), RuntimeValue::Integer(1), true);
    g.define("ST_Y".into(), RuntimeValue::Integer(2), true);
    g.define("ST_WIDTH".into(), RuntimeValue::Integer(3), true);
    g.define("ST_HEIGHT".into(), RuntimeValue::Integer(4), true);
    g.define("ST_CLWIDTH".into(), RuntimeValue::Integer(5), true);
    g.define("ST_CLHEIGHT".into(), RuntimeValue::Integer(6), true);
    g.define("ST_ICON".into(), RuntimeValue::Integer(7), true);
    g.define("ST_MAXIMIZED".into(), RuntimeValue::Integer(8), true);
    g.define("ST_VISIBLE".into(), RuntimeValue::Integer(9), true);
    g.define("ST_ACTIVE".into(), RuntimeValue::Integer(10), true);

    // Dialog Constants
    g.define("BTN_OK".into(), RuntimeValue::Integer(1), true);
    g.define("BTN_CANCEL".into(), RuntimeValue::Integer(2), true);
    g.define("BTN_ABORT".into(), RuntimeValue::Integer(3), true);
    g.define("BTN_RETRY".into(), RuntimeValue::Integer(4), true);
    g.define("BTN_IGNORE".into(), RuntimeValue::Integer(5), true);
    g.define("BTN_YES".into(), RuntimeValue::Integer(6), true);
    g.define("BTN_NO".into(), RuntimeValue::Integer(7), true);

    // Time Constants (Placeholders for now, updated by GETTIME in a real impl)
    g.define("G_TIME_YY".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_MM".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_DD".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_HH".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_NN".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_SS".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_ZZ".into(), RuntimeValue::Integer(0), false);
    g.define("G_TIME_WW".into(), RuntimeValue::Integer(0), false);
    
    // Image Search Constants
    g.define("G_IMG_X".into(), RuntimeValue::Integer(-1), false);
    g.define("G_IMG_Y".into(), RuntimeValue::Integer(-1), false);
    g.define("G_IMG_ID".into(), RuntimeValue::Integer(0), false);

    // Boolean Constants
    g.define("TRUE".into(), RuntimeValue::Bool(true), true);
    g.define("FALSE".into(), RuntimeValue::Bool(false), true);

    // HASHTBL Constants
    g.define("HASH_CASECARE".into(), RuntimeValue::Integer(0x1000), true);
    g.define("HASH_SORT".into(), RuntimeValue::Integer(0x2000), true);
    
    // Time constants (§104)
    g.define("G_TIME_YY".into(), RuntimeValue::Integer(0), true);
    g.define("G_TIME_MM".into(), RuntimeValue::Integer(1), true);
    g.define("G_TIME_DD".into(), RuntimeValue::Integer(2), true);
    g.define("G_TIME_HH".into(), RuntimeValue::Integer(3), true);
    g.define("G_TIME_NN".into(), RuntimeValue::Integer(4), true);
    g.define("G_TIME_SS".into(), RuntimeValue::Integer(5), true);
    g.define("G_TIME_ZZ".into(), RuntimeValue::Integer(6), true);
    g.define("G_TIME_WW".into(), RuntimeValue::Integer(7), true);
    g.define("G_TIME_YY2".into(), RuntimeValue::Integer(8), true);
    g.define("G_TIME_MM2".into(), RuntimeValue::Integer(9), true);
    g.define("G_TIME_DD2".into(), RuntimeValue::Integer(10), true);
    g.define("G_TIME_HH2".into(), RuntimeValue::Integer(11), true);
    g.define("G_TIME_NN2".into(), RuntimeValue::Integer(12), true);
    g.define("G_TIME_SS2".into(), RuntimeValue::Integer(13), true);
    g.define("G_TIME_ZZ2".into(), RuntimeValue::Integer(14), true);
    g.define("G_TIME_WW2".into(), RuntimeValue::Integer(15), true);
    
    // GETID Special Constants
    g.define("GET_ACTIVE_WIN".into(), RuntimeValue::String("GET_ACTIVE_WIN".into()), true);
}
