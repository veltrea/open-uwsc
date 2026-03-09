// Integration Test for PRINT and Arithmetic Operators in Evaluator
mod value;
mod environment;
mod evaluator;
mod parser;
mod lexer;
mod builtins;

use value::RuntimeValue;
use evaluator::Evaluator;
use parser::{Stmt, Expr, Literal, BinaryOp};
use builtins::AiConfig;
use std::collections::HashMap;

fn main() {
    println!("--- Testing PRINT and Arithmetic Operators via Evaluator ---");
    
    let mut evaluator = Evaluator::new(
        AiConfig { api_key: "".into(), model: "".into(), base_url: "".into() },
        HashMap::new()
    );

    // Test Case 1: PRINT 10 + 20
    println!("Expected: 30");
    evaluator.eval_stmts(&[
        Stmt::Print(Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(10.0))),
            BinaryOp::Add,
            Box::new(Expr::Literal(Literal::Number(20.0)))
        ))
    ]);

    // Test Case 2: PRINT 10 / 3 (Float)
    println!("Expected: 3.3333333333333335");
    evaluator.eval_stmts(&[
        Stmt::Print(Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(10.0))),
            BinaryOp::Div,
            Box::new(Expr::Literal(Literal::Number(3.0)))
        ))
    ]);

    // Test Case 3: PRINT "Result: " + (10 MOD 3)
    println!("Expected: Result: 1");
    evaluator.eval_stmts(&[
        Stmt::Print(Expr::Binary(
            Box::new(Expr::Literal(Literal::String("Result: ".into()))),
            BinaryOp::Plus, // Using Plus as string concat
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Literal::Number(10.0))),
                BinaryOp::Mod,
                Box::new(Expr::Literal(Literal::Number(3.0)))
            ))
        ))
    ]);

    // Test Case 4: PRINT TRUE (Boolean Stringification)
    println!("Expected: TRUE");
    evaluator.eval_stmts(&[
        Stmt::Print(Expr::Literal(Literal::Boolean(true)))
    ]);

    println!("--- Integration Test Complete ---");
}
