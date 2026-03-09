use open_uwsc::{lexer, parser, evaluator, builtins, environment, resolver, value};
use builtins::AiConfig;
use std::env;
use std::collections::HashMap;

fn is_admin() -> bool {
    true
}

#[tokio::main]
async fn main() {
    if !is_admin() {
        println!("Insufficient privileges.");
        return;
    }

    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        r#"
            PRINT "Verifying V3 Foundation..."
            DIM x = 10
            PRINT "x = " + x
        "#.to_string()
    };

    let mut lexer = lexer::Lexer::new(&input);
    let mut tokens = Vec::new();
    loop {
        let (token, line) = lexer.next_token();
        tokens.push((token.clone(), line));
        if token == lexer::Token::EOF { break; }
    }

    let mut parser = parser::Parser::new(tokens);
    let stmts = parser.parse();
    
    let mut resolver = open_uwsc::resolver::Resolver::new();
    let locals = resolver.resolve(&stmts);

    let globals = std::sync::Arc::new(std::sync::Mutex::new(environment::Environment::new()));
    let mut evaluator = evaluator::Evaluator::new(globals.clone(), globals, locals, AiConfig::default());
    
    // Map statement IDs to line numbers
    for (stmt_id, line) in parser.stmt_lines {
        evaluator.stmt_lines.insert(stmt_id, line);
    }

    match evaluator.eval_stmts(&stmts) {
        Ok(evaluator::ControlFlow::ExitExit(code)) => {
            let exit_code = if let Some(v) = code {
                v.as_i64() as i32
            } else {
                0
            };
            std::process::exit(exit_code);
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("Execution Error: {}", e);
        }
    }
}
