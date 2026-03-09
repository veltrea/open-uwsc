use open_uwsc::lexer::Lexer;
use open_uwsc::parser::Parser;
use open_uwsc::evaluator::Evaluator;
use open_uwsc::environment::Environment;
use open_uwsc::builtins::AiConfig;
use open_uwsc::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn make_evaluator(input: &str) -> (Evaluator, Vec<open_uwsc::parser::Stmt>) {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok == open_uwsc::lexer::Token::EOF { break; }
        tokens.push(tok);
    }
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let globals = Arc::new(Mutex::new(Environment::new()));
    let evaluator = Evaluator::new(globals, HashMap::new(), AiConfig::default());
    (evaluator, stmts)
}

#[test]
fn test_debug_step_limit() {
    // ステップ制限テスト: max_steps=2 でループが止まることを確認
    let input = "i = 0\nWHILE i < 5\n    i = i + 1\nWEND";
    let (mut evaluator, stmts) = make_evaluator(input);
    evaluator.max_steps = 2;

    let res = evaluator.eval_stmts(&stmts);
    assert!(res.is_err(), "Expected step limit error");
    assert!(
        res.as_ref().unwrap_err().contains("Step limit exceeded"),
        "Unexpected error: {}",
        res.unwrap_err()
    );
}

#[test]
fn test_debug_normal_completion() {
    // 通常完了テスト: max_steps 十分なら最後まで実行されることを確認
    let input = "i = 0\nWHILE i < 5\n    i = i + 1\nWEND";
    let (mut evaluator, stmts) = make_evaluator(input);
    evaluator.max_steps = 100;

    let res = evaluator.eval_stmts(&stmts);
    assert!(res.is_ok());

    let val = evaluator.globals.lock().unwrap().get("i").unwrap();
    assert_eq!(val.as_i64(), 5);
}
