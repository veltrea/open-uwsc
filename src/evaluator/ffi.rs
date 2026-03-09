use crate::parser::{Expr, Stmt};
use crate::value::RuntimeValue;
use crate::evaluator::{Evaluator, ControlFlow};
use crate::builtins::DllFunction;

impl Evaluator {
    pub(crate) fn exec_call(&mut self, path_expr: &Expr) -> ControlFlow {
        let path_val = match self.eval_expr(path_expr) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        let path = path_val.to_string();
        match std::fs::read_to_string(&path) {
            Ok(source) => {
                let mut lexer = crate::lexer::Lexer::new(&source);
                let mut tokens = Vec::new();
                loop {
                    let t = lexer.next_token();
                    let is_eof = t.0 == crate::lexer::Token::EOF;
                    tokens.push(t);
                    if is_eof { break; }
                }
                let mut parser = crate::parser::Parser::new(tokens);
                let stmts = parser.parse();
                self.exec_stmts(&stmts)
            }
            Err(e) => ControlFlow::Error(format!("CALL failed: {} - {}", path, e)),
        }
    }

    pub(crate) fn exec_def_dll(&mut self, name: &String, params: &Vec<crate::parser::Parameter>, ret_type: &String, dll_path: &String) -> ControlFlow {
        self.builtins.dll_registry.insert(name.to_uppercase(), DllFunction {
            name: name.clone(),
            params: params.clone(),
            ret_type: ret_type.clone(),
            dll_path: dll_path.clone(),
        });
        ControlFlow::None
    }

    pub(crate) fn exec_thread(&mut self, expr: &Expr) -> ControlFlow {
        let mut thread_eval = self.clone_for_thread();
        let expr_clone = expr.clone();
        std::thread::spawn(move || {
            let _ = thread_eval.eval_expr(&expr_clone);
        });
        ControlFlow::None
    }
}
