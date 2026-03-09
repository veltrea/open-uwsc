use open_uwsc::evaluator::Evaluator;
use open_uwsc::environment::Environment;
use open_uwsc::builtins::AiConfig;
use open_uwsc::parser::Parser;
use open_uwsc::lexer::Lexer;
use open_uwsc::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn setup_eval() -> Evaluator {
    let mut globals = Environment::new();
    let globals_arc = Arc::new(Mutex::new(globals));
    Evaluator::new(globals_arc, HashMap::new(), AiConfig::default())
}

fn eval_script(eval: &mut Evaluator, script: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(script);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = matches!(tok, open_uwsc::lexer::Token::EOF);
        tokens.push(tok);
        if is_eof { break; }
    }
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    eval.eval_stmts(&stmts)
}

fn get_global(eval: &Evaluator, name: &str) -> RuntimeValue {
    eval.globals.lock().unwrap().get(name).unwrap_or(RuntimeValue::Empty)
}

#[test]
fn test_builtin_copy() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = COPY("ABCDE", 2, 3) 
        PUBLIC T2 = COPY("ABCDE", 4)
        PUBLIC T3 = COPY("ABCDE", 10, 2)
        PUBLIC T4 = COPY("ABCDE", -1, 2)
        PUBLIC T5 = COPY("ABCDE", 2, -1)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // /// Ref: [ORIGINAL_LANGUAGE_SPEC.md] COPY( 文字列, 開始位置, [文字数] )
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::String("BCD".into()));
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::String("DE".into()));
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::String("".into())); // Start beyond length
    assert_eq!(get_global(&eval, "T4"), RuntimeValue::String("".into())); // Negative start
    assert_eq!(get_global(&eval, "T5"), RuntimeValue::String("".into())); // Negative length
}

#[test]
fn test_builtin_pos() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = POS("C", "ABCDE")
        PUBLIC T2 = POS("X", "ABCDE")
        PUBLIC T3 = POS("B", "ABCB", 2)
        PUBLIC T4 = POS("B", "ABCB", -1)
        PUBLIC T5 = POS("", "ABCDE")
        PUBLIC T6 = POS("B", "ABCB", 0)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // /// Ref: [ORIGINAL_LANGUAGE_SPEC.md] POS( 探す文字列, 元の文字列, [n回目] )
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::Integer(3)); // 1-based index
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::Integer(0)); // Not found
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::Integer(4)); // 2nd occurrence
    assert_eq!(get_global(&eval, "T4"), RuntimeValue::Integer(4)); // 1st from back
    assert_eq!(get_global(&eval, "T5"), RuntimeValue::Integer(0)); // Empty search string
    assert_eq!(get_global(&eval, "T6"), RuntimeValue::Integer(0)); // 0th occurrence
}

#[test]
fn test_builtin_length() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = LENGTH("あいうえお")
        PUBLIC T2 = LENGTH("")
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::Integer(5)); // Character count, not bytes
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::Integer(0));
}

#[test]
fn test_builtin_random() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = RANDOM(10)
        PUBLIC T2 = RANDOM(0)
        PUBLIC T3 = RANDOM(-5)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    let t1 = get_global(&eval, "T1").as_i64();
    assert!(t1 >= 0 && t1 < 10);
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::Integer(0)); // max <= 0 returns 0
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::Integer(0));
}

#[test]
fn test_builtin_abs() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = ABS(-10)
        PUBLIC T2 = ABS(10)
        PUBLIC T3 = ABS(-3.14)
        PUBLIC T4 = ABS("-5")
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::Integer(10));
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::Integer(10));
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::Float(3.14));
    assert_eq!(get_global(&eval, "T4"), RuntimeValue::Float(5.0)); // String is coerced to Float typically, or Integer if perfect parser. Let's assert Float based on standard coercion.
}

#[test]
fn test_builtin_sqrt() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = SQRT(16)
        PUBLIC T2 = SQRT(2.25)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::Float(4.0));
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::Float(1.5));
    
    let err_script = r#"
        PUBLIC T3 = SQRT(-1)
    "#;
    assert!(eval_script(&mut eval, err_script).is_err()); // SQRT of negative throws error
}

#[test]
fn test_builtin_file_io() {
    let mut eval = setup_eval();
    // Use a temp file path or generic path. To avoid actual file creation in simple eval tests,
    // we would ideally mock this or ensure it cleans up. For now we will rely on FOPEN failing 
    // predictably or succeeding with a dummy ID as implemented in the stubs right now, then fix it.
    // Right now FOPEN returns dummy 999.
    let script = r#"
        PUBLIC FID = FOPEN("test_dummy.txt", 2)
        PUBLIC RES_PUT = FPUT(FID, "Hello UWSC")
        PUBLIC RES_GET = FGET(FID, 1)
        PUBLIC RES_CLOSE = FCLOSE(FID)
    "#;
    eval_script(&mut eval, script).unwrap();

    // With current stubs, it just returns dummies. We will enforce proper dummy behavior 
    // or real behavior if the user wanted real I/O.
    // For now, based on implementation_plan:
    // Real behavior check
    assert_eq!(get_global(&eval, "FID"), RuntimeValue::Integer(1)); 
    assert_eq!(get_global(&eval, "RES_PUT"), RuntimeValue::Bool(true));
    assert_eq!(get_global(&eval, "RES_GET"), RuntimeValue::String("Hello UWSC".into()));
    assert_eq!(get_global(&eval, "RES_CLOSE"), RuntimeValue::Bool(true));
    
    std::fs::remove_file("test_dummy.txt").ok();
}

#[test]
fn test_builtin_format() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC T1 = FORMAT(123, 5)
        PUBLIC T2 = FORMAT(12.345, 8, 2)
        PUBLIC T3 = FORMAT("abc", -5)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // FORMAT(value, width, [decimal])
    // 5 width, 123 -> "  123"
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::String("  123".into()));
    // 8 width, 2 dec -> "   12.35"
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::String("   12.35".into()));
    // -5 width, abc -> "abc  " (left align)
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::String("abc  ".into()));
}

#[test]
fn test_builtin_token() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC DIR_STR = "apple,banana,cherry"
        PUBLIC T1 = TOKEN(",", DIR_STR)
        PUBLIC T2 = TOKEN(",", DIR_STR)
        PUBLIC T3 = DIR_STR
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // First token is apple, original string becomes banana,cherry
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::String("apple".into()));
    // Second token is banana, original string becomes cherry
    assert_eq!(get_global(&eval, "T2"), RuntimeValue::String("banana".into()));
    // The rest of the string
    assert_eq!(get_global(&eval, "T3"), RuntimeValue::String("cherry".into()));
}

#[test]
fn test_builtin_split_join() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC ARR = SPLIT("A,B,C", ",")
        PUBLIC T1 = JOIN(ARR, "-")
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // Array creation
    let arr_val = get_global(&eval, "ARR");
    match arr_val {
        RuntimeValue::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], RuntimeValue::String("A".into()));
            assert_eq!(items[1], RuntimeValue::String("B".into()));
            assert_eq!(items[2], RuntimeValue::String("C".into()));
        },
        _ => panic!("SPLIT did not return an array"),
    }
    
    // JOIN back together
    assert_eq!(get_global(&eval, "T1"), RuntimeValue::String("A-B-C".into()));
}

#[test]
fn test_builtin_getdir() {
    let mut eval = setup_eval();
    // Use the tests directory which should exist
    let script = r#"
        PUBLIC COUNT = GETDIR("tests")
        PUBLIC FIRST = GETDIR_FILES[0]
    "#;
    eval_script(&mut eval, script).unwrap();
    
    let count = get_global(&eval, "COUNT").as_i64();
    assert!(count > 0);
    
    let first = get_global(&eval, "FIRST");
    assert!(matches!(first, RuntimeValue::String(_)));
}

#[test]
fn test_builtin_modern_regex() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC B1 = RE_MATCH("^\d+$", "12345")
        PUBLIC B2 = RE_MATCH("^\d+$", "123a5")
        PUBLIC ARR = RE_FIND("\d+", "abc123def456")
        PUBLIC S1 = RE_REPLACE("\d+", "NUM", "ID: 123")
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "B1"), RuntimeValue::Bool(true));
    assert_eq!(get_global(&eval, "B2"), RuntimeValue::Bool(false));
    
    let arr = get_global(&eval, "ARR");
    match arr {
        RuntimeValue::Array(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], RuntimeValue::String("123".into()));
            assert_eq!(items[1], RuntimeValue::String("456".into()));
        }
        _ => panic!("RE_FIND should return an array"),
    }
    
    assert_eq!(get_global(&eval, "S1"), RuntimeValue::String("ID: NUM".into()));
}

#[test]
fn test_builtin_modern_json() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC JSON_STR = "{<#DBL>name<#DBL>: <#DBL>Alice<#DBL>, <#DBL>age<#DBL>: 30, <#DBL>tags<#DBL>: [<#DBL>rust<#DBL>, <#DBL>uwsc<#DBL>]}"
        PUBLIC DATA = JSON_PARSE(JSON_STR)
        PUBLIC NAME = DATA["name"]
        PUBLIC TAG1 = DATA["tags"][0]
        PUBLIC STR_BACK = JSON_STRINGIFY(DATA)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "NAME"), RuntimeValue::String("Alice".into()));
    assert_eq!(get_global(&eval, "TAG1"), RuntimeValue::String("rust".into()));
    
    let str_back = get_global(&eval, "STR_BACK").to_string();
    assert!(str_back.contains("\"name\":\"Alice\""));
    assert!(str_back.contains("\"age\":30"));
}

#[test]
fn test_builtin_betweenstr() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC S = "ID: 123, NAME: Alice, ID: 456, NAME: Bob"
        PUBLIC R1 = BETWEENSTR(S, "ID: ", ",")
        PUBLIC R2 = BETWEENSTR(S, "ID: ", ",", 2)
        PUBLIC R3 = BETWEENSTR(S, "NAME: ", ", ID:")
        PUBLIC R4 = BETWEENSTR(S, "NOTFOUND", "ABC")
    "#;
    eval_script(&mut eval, script).unwrap();
    
    assert_eq!(get_global(&eval, "R1"), RuntimeValue::String("123".into()));
    assert_eq!(get_global(&eval, "R2"), RuntimeValue::String("456".into()));
    assert_eq!(get_global(&eval, "R3"), RuntimeValue::String("Alice".into()));
    assert_eq!(get_global(&eval, "R4"), RuntimeValue::String("".into()));
}

#[test]
fn test_builtin_array_advanced() {
    let mut eval = setup_eval();
    let script = r#"
        PUBLIC A1 = SPLIT("C,A,B", ",")
        QSORT(A1)
        
        PUBLIC A2 = SPLIT("1,2,3", ",")
        RESIZE(A2, 5)
        PUBLIC LEN2_5 = LENGTH(A2)
        RESIZE(A2, 2)
        PUBLIC LEN2_2 = LENGTH(A2)
        
        PUBLIC A3 = SPLIT("X,Y", ",")
        SETCLEAR(A3, "Z")
        
        PUBLIC A4 = SPLIT("0,1,2,3,4,5", ",")
        PUBLIC SL = SLICE(A4, 2, 4)
    "#;
    eval_script(&mut eval, script).unwrap();
    
    // QSORT: [A, B, C]
    assert_eq!(get_global(&eval, "A1"), RuntimeValue::Array(vec![
        RuntimeValue::String("A".into()),
        RuntimeValue::String("B".into()),
        RuntimeValue::String("C".into()),
    ]));
    
    // RESIZE
    assert_eq!(get_global(&eval, "LEN2_5"), RuntimeValue::Integer(5));
    assert_eq!(get_global(&eval, "LEN2_2"), RuntimeValue::Integer(2));
    
    // SETCLEAR
    assert_eq!(get_global(&eval, "A3"), RuntimeValue::Array(vec![
        RuntimeValue::String("Z".into()),
        RuntimeValue::String("Z".into()),
    ]));
    
    // SLICE: index 2 to 4 (exclusive of 4 usually or inclusive? original SLICE is SLICE(array, start, [end]))
    // If it follows modern SLICE, it might be [index 2, index 3]
    assert_eq!(get_global(&eval, "SL"), RuntimeValue::Array(vec![
        RuntimeValue::String("2".into()),
        RuntimeValue::String("3".into()),
    ]));
}

#[test]
fn test_builtin_file_handle_io() {
    let mut eval = setup_eval();
    let file_path = "tests/tmp_test_file.txt";
    let script = format!(r#"
        PUBLIC FID = FOPEN("{}", 0) // 0: F_READ | F_WRITE
        FPUT(FID, "Line1", 1)
        FPUT(FID, "Line2", 2)
        FPUT(FID, "ColValue", 2, 2)
        FCLOSE(FID)
        
        PUBLIC FID2 = FOPEN("{}", 0)
        PUBLIC L1 = FGET(FID2, 1)
        PUBLIC L2C2 = FGET(FID2, 2, 2)
        FCLOSE(FID2)
        
        DELETEFILE("{}")
    "#, file_path, file_path, file_path);
    
    eval_script(&mut eval, &script).unwrap();
    
    assert_eq!(get_global(&eval, "L1"), RuntimeValue::String("Line1".into()));
    assert_eq!(get_global(&eval, "L2C2"), RuntimeValue::String("ColValue".into()));
}



