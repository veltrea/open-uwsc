use uwsc_rs::evaluator::{Evaluator, ControlFlow};
use uwsc_rs::parser::{Stmt, Expr, Literal, BinaryOp};
use uwsc_rs::builtins::AiConfig;
use uwsc_rs::value::RuntimeValue;
use std::collections::HashMap;

fn main() {
    let mut evaluator = Evaluator::new(AiConfig::default(), HashMap::new());
    
    // Test Case 1: Empty WHILE loop with max_steps
    // WHILE TRUE; WEND
    evaluator.max_steps = 100;
    let while_empty = Stmt::While {
        condition: Expr::Literal(Literal::Boolean(true)),
        body: vec![],
        id: 1,
    };
    
    println!("Running Test 1: Empty WHILE TRUE; WEND (max_steps=100)");
    let res = evaluator.exec_stmt(&while_empty);
    match res {
        ControlFlow::Error(e) => {
            println!("Test 1 Pass: Caught expected error: {}", e);
            assert!(e.contains("Step limit exceeded"));
        }
        _ => {
            panic!("Test 1 Fail: Expected Step limit exceeded error, but got {:?}", res);
        }
    }

    // Test Case 2: Standard WHILE loop
    // i = 0; WHILE i < 5; i = i + 1; WEND
    evaluator.step_count = 0;
    evaluator.max_steps = 1000;
    evaluator.globals.borrow_mut().define("i".into(), RuntimeValue::Integer(0), false);
    let while_inc = Stmt::While {
        condition: Expr::Binary(
            Box::new(Expr::Variable("i".into(), 2)),
            BinaryOp::Less,
            Box::new(Expr::Literal(Literal::Number(5.0)))
        ),
        body: vec![
            Stmt::Expression(Expr::Assign(
                "i".into(),
                Box::new(Expr::Binary(
                    Box::new(Expr::Variable("i".into(), 2)),
                    BinaryOp::Add,
                    Box::new(Expr::Literal(Literal::Number(1.0)))
                )),
                2
            ), 3)
        ],
        id: 4,
    };
    
    println!("Running Test 2: i=0; WHILE i < 5; i=i+1; WEND");
    let res = evaluator.exec_stmt(&while_inc);
    assert!(matches!(res, ControlFlow::None));
    let val = evaluator.globals.borrow().get("i").unwrap();
    println!("Test 2 Pass: i = {:?}", val);
    assert_eq!(val, RuntimeValue::Integer(5));

    println!("All tests completed successfully!");
}
