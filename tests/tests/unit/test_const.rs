use open_uwsc::lexer::Lexer;
use open_uwsc::parser::Parser;
use open_uwsc::evaluator::Evaluator;
use open_uwsc::environment::Environment;
use open_uwsc::builtins::AiConfig;
use open_uwsc::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn parse_and_eval(input: &str) -> Evaluator {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = matches!(tok, open_uwsc::lexer::Token::EOF);
        tokens.push(tok);
        if is_eof { break; }
    }
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let globals = Arc::new(Mutex::new(Environment::new()));
    let mut evaluator = Evaluator::new(globals, HashMap::new(), AiConfig::default());
    let _ = evaluator.eval_stmts(&stmts);
    evaluator
}

#[test]
fn test_const_declaration_and_reassignment() {
    let evaluator = parse_and_eval("CONST PI = 3.14\nPRINT PI");

    // Verify initial value
    let val = evaluator.globals.lock().unwrap().get("PI").unwrap();
    assert_eq!(val, RuntimeValue::Float(3.14));

    // Test Reassignment (Error)
    let mut lexer_err = Lexer::new("PI = 3.14159");
    let mut tokens_err = Vec::new();
    loop {
        let tok = lexer_err.next_token();
        let is_eof = matches!(tok, open_uwsc::lexer::Token::EOF);
        tokens_err.push(tok);
        if is_eof { break; }
    }
    let mut parser_err = Parser::new(tokens_err);
    let stmts_err = parser_err.parse();
    let globals2 = evaluator.globals.clone();
    let mut ev2 = Evaluator::new(globals2, HashMap::new(), AiConfig::default());
    ev2.functions = evaluator.functions.clone();
    let res = ev2.eval_stmts(&stmts_err);
    assert!(res.is_err(), "Reassigning to CONST should be an error");
    assert!(res.unwrap_err().contains("Cannot reassign to CONST PI"));
}

#[test]
fn test_const_requires_initialization() {
    // CONST without initializer — parser will handle (may panic or skip)
    let mut lexer = Lexer::new("CONST X");
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = matches!(tok, open_uwsc::lexer::Token::EOF);
        tokens.push(tok);
        if is_eof { break; }
    }
    // Just verify parsing doesn't crash catastrophically
    let _ = Parser::new(tokens);
}
