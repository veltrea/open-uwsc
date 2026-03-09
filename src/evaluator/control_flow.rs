use crate::parser::{Expr, Stmt};
use crate::value::RuntimeValue;
use crate::evaluator::{Evaluator, ControlFlow};
use crate::semantics::UwscCompare;
use std::collections::HashMap;

impl Evaluator {
    pub(crate) fn exec_if(&mut self, condition: &Expr, then_branch: &[Stmt], else_if_blocks: &[(Expr, Vec<Stmt>)], else_branch: &Option<Vec<Stmt>>) -> ControlFlow {
        let cond_val = match self.eval_expr(condition) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        if cond_val.is_truthy() {
            return self.exec_stmts(then_branch);
        } else {
            for (elif_condition, elif_branch) in else_if_blocks {
                let elif_val = match self.eval_expr(elif_condition) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };
                if elif_val.is_truthy() {
                    return self.exec_stmts(elif_branch);
                }
            }
            if let Some(else_stmts) = else_branch {
                return self.exec_stmts(else_stmts);
            }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_while(&mut self, condition: &Expr, body: &[Stmt]) -> ControlFlow {
        loop {
            self.step_count += 1;
            if self.max_steps > 0 && self.step_count > self.max_steps {
                return ControlFlow::Error(format!("Step limit exceeded in WHILE: {}", self.max_steps));
            }

            let cond_val = match self.eval_expr(condition) {
                Ok(v) => v,
                Err(e) => return ControlFlow::Error(e),
            };
            if !cond_val.is_truthy() { break; }

            for s in body {
                let cf = self.exec_stmt(s);
                match cf {
                    ControlFlow::Break(n) => {
                        if n <= 1 { return ControlFlow::None; }
                        else { return ControlFlow::Break(n - 1); }
                    }
                    ControlFlow::Continue(n) => {
                        if n <= 1 { break; }
                        else { return ControlFlow::Continue(n - 1); }
                    }
                    ControlFlow::Return(_) | ControlFlow::Error(_) | ControlFlow::Exit | ControlFlow::ExitExit(_) => return cf,
                    ControlFlow::None => {}
                }
            }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_repeat(&mut self, body: &[Stmt], condition: &Expr) -> ControlFlow {
        loop {
            for s in body {
                let cf = self.exec_stmt(s);
                match cf {
                    ControlFlow::Break(n) => {
                        if n <= 1 { return ControlFlow::None; }
                        else { return ControlFlow::Break(n - 1); }
                    }
                    ControlFlow::Continue(n) => {
                        if n <= 1 { break; }
                        else { return ControlFlow::Continue(n - 1); }
                    }
                    ControlFlow::Return(_) | ControlFlow::Error(_) | ControlFlow::Exit | ControlFlow::ExitExit(_) => return cf,
                    ControlFlow::None => {}
                }
            }

            self.step_count += 1;
            if self.max_steps > 0 && self.step_count > self.max_steps {
                return ControlFlow::Error(format!("Step limit exceeded in REPEAT: {}", self.max_steps));
            }

            let cond_val = match self.eval_expr(condition) {
                Ok(v) => v,
                Err(e) => return ControlFlow::Error(e),
            };
            if cond_val.is_truthy() { break; }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_select(&mut self, target: &Expr, cases: &[(Vec<Expr>, Vec<Stmt>)], default_branch: &Option<Vec<Stmt>>) -> ControlFlow {
        let target_val = match self.eval_expr(target) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };

        let mut matched = false;
        for (case_exprs, body) in cases {
            for expr in case_exprs {
                let case_val = match self.eval_expr(expr) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Error(e),
                };
                if target_val.compare(&case_val, crate::parser::CmpOp::Eq) == Ok(RuntimeValue::Bool(true)) {
                    matched = true;
                    break;
                }
            }
            if matched {
                for s in body {
                    let cf = self.exec_stmt(s);
                    if !matches!(cf, ControlFlow::None) { return cf; }
                }
                return ControlFlow::None;
            }
        }

        if !matched {
            if let Some(default_stmts) = default_branch {
                for s in default_stmts {
                    let cf = self.exec_stmt(s);
                    if !matches!(cf, ControlFlow::None) { return cf; }
                }
            }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_try(&mut self, try_branch: &[Stmt], except_branch: &Option<Vec<Stmt>>, finally_branch: &Option<Vec<Stmt>>) -> ControlFlow {
        let mut cf = ControlFlow::None;
        
        for s in try_branch {
            cf = self.exec_stmt(s);
            if !matches!(cf, ControlFlow::None) { break; }
        }

        if let ControlFlow::Error(ref e) = cf {
            self.last_error_msg = e.clone(); 
            self.caught_error_msg = e.clone();
            self.caught_error_line = self.last_error_line;
            if let Some(except_stmts) = except_branch {
                cf = ControlFlow::None;
                for s in except_stmts {
                    let res = self.exec_stmt(s);
                    if !matches!(res, ControlFlow::None) {
                        cf = res;
                        break;
                    }
                }
            }
        }

        if let Some(finally_stmts) = finally_branch {
            let mut finally_cf = ControlFlow::None;
            for s in finally_stmts {
                finally_cf = self.exec_stmt(s);
                if !matches!(finally_cf, ControlFlow::None) { break; }
            }
            if !matches!(finally_cf, ControlFlow::None) {
                cf = finally_cf;
            }
        }

        cf
    }

    pub(crate) fn exec_for(&mut self, var: &String, from: &Expr, to: &Expr, step: &Option<Expr>, body: &[Stmt]) -> ControlFlow {
        let start_val = match self.eval_expr(from) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        let end_val = match self.eval_expr(to) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        
        let from_f = start_val.as_f64();
        let to_f = end_val.as_f64();

        let step_f = if let Some(s) = step {
            match self.eval_expr(s) {
                Ok(v) => {
                    let f = v.as_f64();
                    if f == 0.0 { return ControlFlow::Error("FOR loop step cannot be zero".into()); }
                    f
                }
                Err(e) => return ControlFlow::Error(e),
            }
        } else {
            if from_f <= to_f { 1.0 } else { -1.0 }
        };

        let mut current_f = from_f;
        let initial_val = if current_f.fract() == 0.0 {
            RuntimeValue::Integer(current_f as i64)
        } else {
            RuntimeValue::Float(current_f)
        };
        self.env.lock().unwrap().define(var.clone(), initial_val, false);

        loop {
            if step_f > 0.0 && current_f > to_f { break; }
            if step_f < 0.0 && current_f < to_f { break; }

            self.step_count += 1;
            if self.max_steps > 0 && self.step_count > self.max_steps {
                return ControlFlow::Error(format!("Step limit exceeded in FOR: {}", self.max_steps));
            }

            for s in body {
                let cf = self.exec_stmt(s);
                match cf {
                    ControlFlow::Break(n) => {
                        if n <= 1 { return ControlFlow::None; }
                        else { return ControlFlow::Break(n - 1); }
                    }
                    ControlFlow::Continue(n) => {
                        if n <= 1 { break; }
                        else { return ControlFlow::Continue(n - 1); }
                    }
                    ControlFlow::Return(_) | ControlFlow::Error(_) | ControlFlow::Exit | ControlFlow::ExitExit(_) => return cf,
                    ControlFlow::None => {}
                }
            }

            current_f += step_f;
            let next_val = if current_f.fract() == 0.0 {
                RuntimeValue::Integer(current_f as i64)
            } else {
                RuntimeValue::Float(current_f)
            };

            if let Err(e) = self.env.lock().unwrap().assign(var, next_val) {
                return ControlFlow::Error(e);
            }
        }
        ControlFlow::None
    }

    pub(crate) fn exec_with(&mut self, expr: &Expr, body: &[Stmt]) -> ControlFlow {
        let val = match self.eval_expr(expr) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        self.with_stack.push(val);
        let res = self.exec_stmts(body);
        self.with_stack.pop();
        res
    }

    pub(crate) fn exec_module(&mut self, name: &String, body: &[Stmt]) -> ControlFlow {
        // MODULE execution: execute body in a separate environment-like scope
        // and register all declared publics/dims into self.modules[name]
        
        let local_globals = std::sync::Arc::new(std::sync::Mutex::new(crate::environment::Environment::new()));
        let mut mod_eval = Evaluator::new(local_globals.clone(), self.globals.clone(), self.locals.clone(), self.ai.clone());
        mod_eval.functions = self.functions.clone();
        mod_eval.modules = self.modules.clone();
        
        if let Err(e) = mod_eval.eval_stmts(body) {
            return ControlFlow::Error(format!("Module '{}' initialization failed: {}", name, e));
        }
        
        // Register module functions with prefix (uppercase module name)
        let name_up = name.to_uppercase();
        for (f_name, f_data) in mod_eval.functions {
            // Only add functions that were defined inside this module (not inherited)
            if !self.functions.contains_key(&f_name) {
                self.functions.insert(format!("{}.{}", name_up, f_name), f_data);
            }
        }
        
        self.modules.insert(name_up, local_globals);
        ControlFlow::None
    }

    pub(crate) fn exec_enum(&mut self, name: &String, members_def: &Vec<(String, Option<Expr>)>) -> ControlFlow {
        let env_arc = std::sync::Arc::new(std::sync::Mutex::new(crate::environment::Environment::new()));
        let mut next_val = 0;
        {
            let mut env = env_arc.lock().unwrap();
            for (m_name, m_expr) in members_def {
                if let Some(expr) = m_expr {
                    match self.eval_expr(expr) {
                        Ok(v) => {
                            next_val = v.as_i64();
                        }
                        Err(e) => return ControlFlow::Error(e),
                    }
                }
                env.define(m_name.to_uppercase(), RuntimeValue::Integer(next_val), true);
                next_val += 1;
            }
        }
        self.modules.insert(name.to_uppercase(), env_arc);
        ControlFlow::None
    }
}
