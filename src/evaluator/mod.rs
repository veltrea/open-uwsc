use crate::parser::{Expr, Stmt, BinaryOp, Literal, AssignOp};
use crate::value::RuntimeValue;
use crate::environment::Environment;
use crate::builtins::{AiConfig, Builtins};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, GetCursorPos};
use windows::Win32::Foundation::POINT;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum ControlFlow {
    None,
    Break(usize),
    Continue(usize),
    Return(RuntimeValue),
    Exit,      // Exit current function or script
    ExitExit(Option<RuntimeValue>),  // Terminate entire program with optional code
    Error(String),
}

pub struct Evaluator {
    pub env: Arc<Mutex<Environment>>,
    pub globals: Arc<Mutex<Environment>>,
    pub locals: HashMap<usize, usize>, // stmt_id -> local_idx
    pub step_count: usize,
    pub max_steps: usize,
    pub ai: AiConfig,
    pub breakpoints: std::collections::HashSet<usize>,
    pub functions: HashMap<String, (Vec<crate::parser::Parameter>, Vec<Stmt>)>,
    pub builtins: Builtins,
    pub explicit_declaration: bool,
    pub with_stack: Vec<RuntimeValue>,
    pub modules: HashMap<String, Arc<Mutex<Environment>>>,
    pub last_error_msg: String,
    pub last_error_line: usize,
    pub caught_error_msg: String,
    pub caught_error_line: usize,
    pub stmt_lines: HashMap<usize, usize>, // stmt_id -> line_number
}

mod constants;
mod expression;
mod control_flow;
mod ffi;
mod experimental;

impl Evaluator {
    pub fn new(env: Arc<Mutex<Environment>>, globals: Arc<Mutex<Environment>>, locals: HashMap<usize, usize>, ai: AiConfig) -> Self {
        // Define Constants via submodule
        if Arc::ptr_eq(&env, &globals) {
            constants::init(&mut globals.lock().unwrap());
        }

        Self {
            env,
            globals,
            locals,
            step_count: 0,
            max_steps: 0,
            ai: ai.clone(),
            breakpoints: std::collections::HashSet::new(),
            functions: HashMap::new(),
            builtins: Builtins::new(None, ai),
            explicit_declaration: false,
            with_stack: Vec::new(),
            modules: HashMap::new(),
            last_error_msg: String::new(),
            last_error_line: 0,
            caught_error_msg: String::new(),
            caught_error_line: 0,
            stmt_lines: HashMap::new(),
        }
    }

    pub fn clone_for_thread(&self) -> Self {
        Self {
            env: self.globals.clone(), // Start thread in global scope
            globals: self.globals.clone(),
            locals: self.locals.clone(),
            step_count: 0,
            max_steps: self.max_steps,
            ai: self.ai.clone(),
            breakpoints: self.breakpoints.clone(),
            functions: self.functions.clone(),
            builtins: Builtins::new(None, self.ai.clone()),
            explicit_declaration: self.explicit_declaration,
            with_stack: self.with_stack.clone(),
            modules: self.modules.clone(),
            last_error_msg: self.last_error_msg.clone(),
            last_error_line: self.last_error_line,
            caught_error_msg: self.caught_error_msg.clone(),
            caught_error_line: self.caught_error_line,
            stmt_lines: self.stmt_lines.clone(),
        }
    }

    pub fn eval_stmts(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, String> {
        // Pre-scan for functions and procedures
        for stmt in stmts {
            self.register_function(stmt);
        }

        match self.exec_stmts(stmts) {
            ControlFlow::None => Ok(ControlFlow::None),
            ControlFlow::Exit => Ok(ControlFlow::Exit),
            ControlFlow::ExitExit(code) => Ok(ControlFlow::ExitExit(code)),
            ControlFlow::Return(v) => Ok(ControlFlow::Return(v)),
            ControlFlow::Break(n) => Err(format!("Unexpected break outside loop (depth: {})", n)),
            ControlFlow::Continue(n) => Err(format!("Unexpected continue outside loop (depth: {})", n)),
            ControlFlow::Error(e) => Err(e),
        }
    }

    fn register_function(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Procedure { name, params, body } => {
                self.functions.insert(name.to_uppercase(), (params.clone(), body.clone()));
            }
            Stmt::Function { name, params, body } => {
                self.functions.insert(name.to_uppercase(), (params.clone(), body.clone()));
            }
            _ => {}
        }
    }

    pub(crate) fn exec_stmts(&mut self, stmts: &[Stmt]) -> ControlFlow {
        for stmt in stmts {
            let cf = self.exec_stmt(stmt);
            if !matches!(cf, ControlFlow::None) {
                return cf;
            }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_stmt(&mut self, stmt: &Stmt) -> ControlFlow {
        // --- Pre-Execution Logic (Debug, Measurements) ---
        if let Some(cf) = self.pre_execute(stmt) {
            return cf;
        }

        // --- Experimental Layer Dispatch ---
        if let Some(cf) = self.exec_experimental(stmt) {
            return cf;
        }

        let stmt_id = match stmt {
            Stmt::Expression(_, id) => Some(*id),
            Stmt::If { id, .. } => Some(*id),
            Stmt::While { id, .. } => Some(*id),
            Stmt::Repeat { id, .. } => Some(*id),
            Stmt::Select { id, .. } => Some(*id),
            Stmt::Try { id, .. } => Some(*id),
            Stmt::For { id, .. } => Some(*id),
            Stmt::ForIn { id, .. } => Some(*id),
            Stmt::Print(_, id) => Some(*id),
            Stmt::Dim { id, .. } => Some(*id),
            Stmt::Public { id, .. } => Some(*id),
            Stmt::Hashtbl(_, _, id) => Some(*id),
            Stmt::TextBlock { id, .. } => Some(*id),
            Stmt::Const(_, _, id) => Some(*id),
            Stmt::Module { id, .. } => Some(*id),
            Stmt::Enum { id, .. } => Some(*id),
            Stmt::With { id, .. } => Some(*id),
            Stmt::Exit(id) => Some(*id),
            Stmt::ExitExit(_, id) => Some(*id),
            _ => None,
        };

        if let Some(id) = stmt_id {
            if let Some(&line) = self.stmt_lines.get(&id) {
                self.last_error_line = line;
            }
        }

        let cf = match stmt {
            Stmt::Expression(expr, _) => {
                match self.eval_expr(expr) {
                    Ok(_) => ControlFlow::None,
                    Err(e) => ControlFlow::Error(e),
                }
            }
            Stmt::With { expr, body, .. } => {
                self.exec_with(expr, body)
            }
            Stmt::Print(expr, _) => {
                match self.eval_expr(expr) {
                    Ok(val) => {
                        println!("{}", val);
                        ControlFlow::None
                    }
                    Err(e) => ControlFlow::Error(e),
                }
            }
            Stmt::If { condition, then_branch, else_if_blocks, else_branch, .. } => {
                self.exec_if(condition, then_branch, else_if_blocks, else_branch)
            }
            Stmt::While { condition, body, .. } => {
                self.exec_while(condition, body)
            }
            Stmt::Repeat { body, condition, .. } => {
                self.exec_repeat(body, condition)
            }
            Stmt::Select { target, cases, default_branch, .. } => {
                self.exec_select(target, cases, default_branch)
            }
            Stmt::Try { try_branch, except_branch, finally_branch, .. } => {
                self.exec_try(try_branch, except_branch, finally_branch)
            }
            Stmt::Break(n) => ControlFlow::Break(n.unwrap_or(1)),
            Stmt::Continue(n) => ControlFlow::Continue(n.unwrap_or(1)),
            Stmt::Dim { name, init, dimensions, .. } => {
                if !dimensions.is_empty() {
                    let mut dims = Vec::new();
                    for d in dimensions {
                        match self.eval_expr(d) {
                            Ok(v) => dims.push(v.as_i64()),
                            Err(e) => return ControlFlow::Error(e),
                        }
                    }
                    // DIM arr[5] in UWSC creates 6 elements (0..5)
                    // So we need to add 1 to each dimension
                    let dims_plus_one: Vec<i64> = dims.iter().map(|d| d + 1).collect();
                    let val = self.create_multi_dim_array(&dims_plus_one);
                    self.env.lock().unwrap().define(name.to_uppercase(), val, false);
                } else {
                    let val = if let Some(e) = init {
                        match self.eval_expr(e) {
                            Ok(v) => v,
                            Err(e) => return ControlFlow::Error(e),
                        }
                    } else {
                        RuntimeValue::Empty
                    };
                    self.env.lock().unwrap().define(name.to_uppercase(), val, false);
                }
                ControlFlow::None
            }
            Stmt::Exit(_) => ControlFlow::Exit,
            Stmt::ExitExit(code_expr, _) => {
                let code = if let Some(expr) = code_expr {
                    match self.eval_expr(expr) {
                        Ok(v) => Some(v),
                        Err(e) => return ControlFlow::Error(e),
                    }
                } else {
                    None
                };
                ControlFlow::ExitExit(code)
            }
            Stmt::Call(path_expr, _) => {
                self.exec_call(path_expr)
            }
            Stmt::Hashtbl(name, init, _) => {
                let case_care = if let Some(init_expr) = init {
                    match self.eval_expr(init_expr) {
                        Ok(v) => (v.as_i64() & 0x1000) != 0,
                        Err(e) => return ControlFlow::Error(e),
                    }
                } else {
                    false
                };
                self.env.lock().unwrap().define(name.to_uppercase(), RuntimeValue::Hash { map: HashMap::new(), case_care }, false);
                ControlFlow::None
            }
            Stmt::TextBlock { name, content, .. } => {
                self.env.lock().unwrap().define(name.to_uppercase(), RuntimeValue::String(content.clone()), false);
                ControlFlow::None
            }
            Stmt::Public { name, init, dimensions, .. } => {
                if !dimensions.is_empty() {
                    let mut dims = Vec::new();
                    for d in dimensions {
                        match self.eval_expr(d) {
                            Ok(v) => dims.push(v.as_i64()),
                            Err(e) => return ControlFlow::Error(e),
                        }
                    }
                    let dims_plus_one: Vec<i64> = dims.iter().map(|d| d + 1).collect();
                    let val = self.create_multi_dim_array(&dims_plus_one);
                    self.globals.lock().unwrap().define(name.to_uppercase(), val, false);
                } else {
                    let val = if let Some(e) = init {
                        match self.eval_expr(e) {
                            Ok(v) => v,
                            Err(e) => return ControlFlow::Error(e),
                        }
                    } else {
                        RuntimeValue::Empty
                    };
                    self.globals.lock().unwrap().define(name.to_uppercase(), val, false);
                }
                ControlFlow::None
            }
            Stmt::For { var, from, to, step, body, .. } => {
                self.exec_for(var, from, to, step, body)
            }
            Stmt::Const(name, init, _) => {
                let val = match self.eval_expr(init) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };
                self.env.lock().unwrap().define(name.to_uppercase(), val, true);
                ControlFlow::None
            }
            Stmt::DefDll { name, params, ret_type, dll_path } => {
                self.exec_def_dll(name, params, ret_type, dll_path)
            }
            Stmt::Thread(expr) => {
                self.exec_thread(expr)
            }
            Stmt::Option(name, _) => {
                if name.to_uppercase() == "EXPLICIT" {
                    self.explicit_declaration = true;
                }
                ControlFlow::None
            }
            Stmt::ForIn { .. } => {
                // Should be handled by exec_experimental.
                ControlFlow::Error("Experimental ForIn fell through".into())
            }
            Stmt::Module { name, body, .. } => {
                self.exec_module(name, body)
            }
            Stmt::Enum { name, members, .. } => {
                self.exec_enum(name, members)
            }
            Stmt::Procedure { .. } | Stmt::Function { .. } => {
                self.register_function(stmt);
                ControlFlow::None
            }
            Stmt::Return(_) => ControlFlow::None,
        };
        cf
    }

    /// ステートメント実行前の処理 (デバッグ・計測用)
    fn pre_execute(&mut self, stmt: &Stmt) -> Option<ControlFlow> {
        // 1. ブレークポイント判定
        if let Some(stmt_id) = self.get_stmt_id(stmt) {
            if self.breakpoints.contains(&stmt_id) {
                println!("--- [DEBUGGER] Breakpoint hit at ID: {} ---", stmt_id);
                println!("--- Current Variables: ---");
                self.env.lock().unwrap().dump();
                println!("--------------------------");
                println!("Press Enter to continue...");
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
            }
        }

        // 2. ステップ数計測・制限
        self.step_count += 1;
        if self.max_steps > 0 && self.step_count > self.max_steps {
            return Some(ControlFlow::Error(format!("Step limit exceeded: {}", self.max_steps)));
        }

        None
    }

    pub(crate) fn create_multi_dim_array(&self, dims: &[i64]) -> RuntimeValue {
        if dims.is_empty() {
            return RuntimeValue::Empty;
        }
        let size = dims[0] as usize;
        let mut vec = Vec::with_capacity(size);
        if dims.len() == 1 {
            for _ in 0..size {
                vec.push(RuntimeValue::Empty);
            }
        } else {
            for _ in 0..size {
                vec.push(self.create_multi_dim_array(&dims[1..]));
            }
        }
        RuntimeValue::Array(vec)
    }

    fn get_stmt_id(&self, stmt: &Stmt) -> Option<usize> {
        match stmt {
            Stmt::Expression(_, id) => Some(*id),
            Stmt::If { id, .. } => Some(*id),
            Stmt::While { id, .. } => Some(*id),
            Stmt::Repeat { id, .. } => Some(*id),
            Stmt::Select { id, .. } => Some(*id),
            Stmt::Try { id, .. } => Some(*id),
            Stmt::For { id, .. } => Some(*id),
            Stmt::Print(_, id) => Some(*id),
            Stmt::Procedure { .. } | Stmt::Function { .. } => None,
            Stmt::Const(_, _, id) | Stmt::Dim { id, .. } | Stmt::Public { id, .. } |
            Stmt::Exit(id) | Stmt::ExitExit(_, id) | Stmt::Hashtbl(_, _, id) |
            Stmt::TextBlock { id, .. } | Stmt::Call(_, id) |
            Stmt::ForIn { id, .. } | Stmt::Option(_, id) | Stmt::With { id, .. } |
            Stmt::Module { id, .. } | Stmt::Enum { id, .. } => Some(*id),
            Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::DefDll { .. } | Stmt::Thread(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Literal, BinaryOp};

    #[test]
    fn test_eval_print_arithmetic() {
        let globals = Arc::new(Mutex::new(Environment::new()));
        let mut evaluator = Evaluator::new(
            globals.clone(),
            HashMap::new(),
            AiConfig::default(),
        );
        evaluator.max_steps = 100;

        // Standard Add
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(10.0))),
            BinaryOp::Add,
            Box::new(Expr::Literal(Literal::Number(20.0)))
        );
        let val = evaluator.eval_expr(&expr).unwrap();
        assert_eq!(val, RuntimeValue::Integer(30));

        // Div Always Float
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(10.0))),
            BinaryOp::Div,
            Box::new(Expr::Literal(Literal::Number(2.0)))
        );
        let val = evaluator.eval_expr(&expr).unwrap();
        assert_eq!(val, RuntimeValue::Float(5.0));

        // Modulo
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Number(10.0))),
            BinaryOp::Mod,
            Box::new(Expr::Literal(Literal::Number(3.0)))
        );
        let val = evaluator.eval_expr(&expr).unwrap();
        assert_eq!(val, RuntimeValue::Integer(1));

        // String Concat
        let expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::String("Age: ".into()))),
            BinaryOp::Plus,
            Box::new(Expr::Literal(Literal::Number(25.0)))
        );
        let val = evaluator.eval_expr(&expr).unwrap();
        assert_eq!(val, RuntimeValue::String("Age: 25".into()));

        // Assignment (Global)
        evaluator.globals.lock().unwrap().define("x".into(), RuntimeValue::Empty, false);
        let target_expr = Box::new(Expr::Variable("x".into(), 999));
        let assign_expr = Expr::Assign(target_expr, AssignOp::Assign, Box::new(Expr::Literal(Literal::Number(100.0))), 999);
        evaluator.eval_expr(&assign_expr).unwrap();
        let var_expr = Expr::Variable("x".into(), 999);
        assert_eq!(evaluator.eval_expr(&var_expr).unwrap(), RuntimeValue::Integer(100));

        // Logical Operators
        let and_expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::Boolean(true))),
            BinaryOp::And,
            Box::new(Expr::Literal(Literal::Number(1.0)))
        );
        assert_eq!(evaluator.eval_expr(&and_expr).unwrap(), RuntimeValue::Bool(true));
        
        let not_expr = Expr::Unary(crate::parser::UnaryOp::Not, Box::new(Expr::Literal(Literal::Number(0.0))));
        assert_eq!(evaluator.eval_expr(&not_expr).unwrap(), RuntimeValue::Integer(-1));

        // IF Statement & Comparison
        let if_stmt = Stmt::If {
            condition: Expr::Binary(
                Box::new(Expr::Literal(Literal::Number(10.0))),
                BinaryOp::Greater,
                Box::new(Expr::Literal(Literal::Number(5.0)))
            ),
            then_branch: vec![
                Stmt::Print(Expr::Literal(Literal::String("OK".into())), 2000)
            ],
            else_if_blocks: vec![],
            else_branch: None,
            id: 2001,
        };
        evaluator.exec_stmt(&if_stmt);

        let eq_expr = Expr::Binary(
            Box::new(Expr::Literal(Literal::String("10".into()))),
            BinaryOp::Equal,
            Box::new(Expr::Literal(Literal::Number(10.0)))
        );
        assert_eq!(evaluator.eval_expr(&eq_expr).unwrap(), RuntimeValue::Bool(true));
    }
}
