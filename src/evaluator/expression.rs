use crate::parser::{Expr, Literal, BinaryOp, AssignOp};
use crate::value::RuntimeValue;
use crate::environment::Environment; // Kept this as it's used in the original code
use crate::evaluator::{Evaluator, ControlFlow}; // Combined evaluator imports
use crate::semantics::{UwscArithmetic, UwscCompare, UwscLogical, UwscUnary, UwscAssignment}; // Combined semantics imports
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, GetCursorPos};
use windows::Win32::Foundation::POINT;
use std::sync::Arc;

// Moved apply_assign_op to a standalone function to avoid borrow conflicts
fn do_apply_assign_op(target: &mut RuntimeValue, value: RuntimeValue, op: &AssignOp) -> Result<RuntimeValue, String> {
    match op {
        AssignOp::Assign => target.assign(value),
        AssignOp::AddAssign => target.add_assign(&value),
        AssignOp::SubAssign => target.sub_assign(&value),
        AssignOp::MulAssign => target.mul_assign(&value),
        AssignOp::DivAssign => target.div_assign(&value),
        AssignOp::ModAssign => target.mod_assign(&value),
    }
}

impl Evaluator {
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<RuntimeValue, String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => {
                    if n.fract() == 0.0 { Ok(RuntimeValue::Integer(*n as i64)) } 
                    else { Ok(RuntimeValue::Float(*n)) }
                }
                Literal::String(s) => Ok(RuntimeValue::String(s.clone())),
                Literal::Boolean(b) => Ok(RuntimeValue::Bool(*b)),
                Literal::Null => Ok(RuntimeValue::Null),
                Literal::Empty => Ok(RuntimeValue::Empty),
                Literal::SpecialConst(s) => {
                    let val = match s.to_uppercase().as_str() {
                        "CR" => "\r",
                        "LF" => "\n",
                        "TAB" => "\t",
                        "DBL" => "\"",
                        _ => "", // Default or error
                    };
                    Ok(RuntimeValue::String(val.to_string()))
                }
            },
            Expr::Variable(name, id) => {
                let res = self.lookup_variable(name, *id);
                if res == RuntimeValue::Empty {
                    // Check if it's a module name
                    if self.modules.contains_key(&name.to_uppercase()) {
                        // Return something representing the module? 
                        // For now, let's just return Empty but let MemberAccess handle it.
                    }
                }
                Ok(res)
            }
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                let res = match op {
                    BinaryOp::Add | BinaryOp::Plus => l.add(&r),
                    BinaryOp::Sub | BinaryOp::Minus => l.sub(&r),
                    BinaryOp::Mul => l.mul(&r),
                    BinaryOp::Div => l.div(&r),
                    BinaryOp::Mod => l.mod_op(&r),
                    // Comparison Operations (§2.1 / §27)
                    BinaryOp::Equal => l.compare(&r, crate::parser::CmpOp::Eq),
                    BinaryOp::NotEqual => l.compare(&r, crate::parser::CmpOp::Ne),
                    BinaryOp::Greater => l.compare(&r, crate::parser::CmpOp::Gt),
                    BinaryOp::Less => l.compare(&r, crate::parser::CmpOp::Lt),
                    BinaryOp::GreaterEq => l.compare(&r, crate::parser::CmpOp::Ge),
                    BinaryOp::LessEq => l.compare(&r, crate::parser::CmpOp::Le),
                    // Logical/Bitwise Operations (§2.1 / §73)
                    BinaryOp::And => l.and(&r),
                    BinaryOp::Or => l.or(&r),
                    BinaryOp::Xor => l.xor(&r),
                };
                res
            }
            Expr::Unary(op, right) => {
                let r = self.eval_expr(right)?;
                match op {
                    crate::parser::UnaryOp::Not => {
                        // Logical NOT / Bitwise NOT (§2.1 / §73)
                        r.not()
                    }
                    crate::parser::UnaryOp::Neg => {
                        // Arithmetic Negation (§2.1 / §34)
                        r.neg()
                    }
                }
            }
            Expr::Assign(target, op, value, id) => {
                let val = self.eval_expr(value)?;
                self.assign_to_expr(target, val, op, *id)
            }
            Expr::ArrayAccess(array_expr, indices, _) => {
                let mut collection = self.eval_expr(array_expr)?;
                for index_expr in indices {
                    let idx_val = self.eval_expr(index_expr)?;
                    match collection {
                        RuntimeValue::Array(vals) => {
                            let idx = idx_val.as_i64() as usize;
                            if idx < vals.len() {
                                collection = vals[idx].clone();
                            } else {
                                return Err(format!("Array index out of bounds: {}", idx));
                            }
                        }
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if case_care { idx_val.to_string() } else { idx_val.to_string().to_uppercase() };
                            if let Some(val) = map.get(&key) {
                                collection = val.clone();
                            } else {
                                collection = RuntimeValue::Empty; 
                            }
                        }
                        _ => return Err("Cannot index non-collection value".into()),
                    }
                }
                Ok(collection)
            }
            Expr::MemberAccess(expr, member, _) => {
                let left_val = self.eval_expr(expr)?;
                match left_val {
                    RuntimeValue::Hash { map, case_care } => {
                        let key = if case_care { member.clone() } else { member.to_uppercase() };
                        Ok(map.get(&key).cloned().unwrap_or(RuntimeValue::Empty))
                    }
                    _ => {
                        // Try module mapping
                        if let Expr::Variable(m_name, _) = &**expr {
                            if let Some(m_env) = self.modules.get(&m_name.to_uppercase()) {
                                if let Some(val) = m_env.lock().unwrap().get(&member.to_uppercase()) {
                                    return Ok(val);
                                }
                            }
                        }
                        Err(format!("Member '{}' not found or cannot access member of type {}", member, left_val.type_name()))
                    }
                }
            }
            Expr::WithMember(member, _) => {
                for obj in self.with_stack.iter().rev() {
                    match obj {
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if *case_care { member.clone() } else { member.to_uppercase() };
                            if let Some(val) = map.get(&key) {
                                return Ok(val.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Err(format!("Member '{}' not found in WITH stack", member))
            }
            Expr::GlobalVariable(name, _) => {
                let globals = self.globals.lock().unwrap();
                let name_up = name.to_uppercase();
                if let Some(val) = globals.values.get(&name_up) {
                    Ok(val.clone())
                } else {
                    Err(format!("Undefined global variable: {}", name_up))
                }
            }
            Expr::ThisMember(member, _) => {
                // Priority 1: WITH stack
                if let Some(with_obj) = self.with_stack.last() {
                    match with_obj {
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if *case_care { member.clone() } else { member.to_uppercase() };
                            if let Some(val) = map.get(&key) {
                                return Ok(val.clone());
                            }
                        }
                        _ => {}
                    }
                }
                
                // Priority 2: Lexical scope (walk up to find first scope containing the member)
                // In UWSC, THIS within a module function usually refers to the module members.
                let name_up = member.to_uppercase();
                let mut current_env = Some(self.env.clone());
                
                while let Some(env_arc) = current_env {
                    let env = env_arc.lock().unwrap();
                    if let Some(val) = env.values.get(&name_up) {
                        return Ok(val.clone());
                    }
                    // Stop if we reach globals? Actually, THIS should probably stay within module/class boundaries.
                    // For now, let's allow it to hit the first containing scope.
                    current_env = env.outer.clone();
                    if let Some(ref outer) = current_env {
                        if Arc::ptr_eq(outer, &self.globals) {
                            break; // Don't look in root globals for THIS.
                        }
                    } else {
                        break;
                    }
                }

                Err(format!("Member '{}' not found in THIS context", member))
            }
            Expr::ArrayLiteral(elements, _) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(RuntimeValue::Array(vals))
            }
            Expr::Call(callee_expr, args, _) => {
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }

                let (res, func_name_for_sync) = match &**callee_expr {
                    Expr::Variable(name, _) => {
                        (self.call_function_or_builtin(name, &mut eval_args)?, name.to_string())
                    }
                    Expr::MemberAccess(left, member, _) => {
                        if let Expr::Variable(m_name, _) = &**left {
                            let full_name = format!("{}.{}", m_name.to_uppercase(), member.to_uppercase());
                            (self.call_function_or_builtin(&full_name, &mut eval_args)?, full_name)
                        } else {
                            return Err("Nested member call not supported".into());
                        }
                    }
                    _ => return Err("Unexpected call target".into()),
                };

                // Sync special variables and side effects for specific functions
                let name_up = func_name_for_sync.to_uppercase();
                match name_up.as_str() {
                    "CHKIMG" => {
                        let mut env = self.env.lock().unwrap();
                        env.define("G_IMG_X".to_string(), RuntimeValue::Integer(self.builtins.last_img_x as i64), true);
                        env.define("G_IMG_Y".to_string(), RuntimeValue::Integer(self.builtins.last_img_y as i64), true);
                        env.define("G_IMG_ID".to_string(), RuntimeValue::Integer(self.builtins.last_img_id as i64), true);
                    }
                    "GETTIME" => {
                        let mut env = self.env.lock().unwrap();
                        env.define("G_TIME_YY".to_string(), RuntimeValue::Integer(self.builtins.g_time_yy as i64), true);
                        env.define("G_TIME_MM".to_string(), RuntimeValue::Integer(self.builtins.g_time_mm as i64), true);
                        env.define("G_TIME_DD".to_string(), RuntimeValue::Integer(self.builtins.g_time_dd as i64), true);
                        env.define("G_TIME_HH".to_string(), RuntimeValue::Integer(self.builtins.g_time_hh as i64), true);
                        env.define("G_TIME_NN".to_string(), RuntimeValue::Integer(self.builtins.g_time_nn as i64), true);
                        env.define("G_TIME_SS".to_string(), RuntimeValue::Integer(self.builtins.g_time_ss as i64), true);
                        env.define("G_TIME_ZZ".to_string(), RuntimeValue::Integer(self.builtins.g_time_zz as i64), true);
                        env.define("G_TIME_WW".to_string(), RuntimeValue::Integer(self.builtins.g_time_ww as i64), true);
                    }
                    "TOKEN" => {
                        if let Some(rest) = self.builtins.last_token_remaining.take() {
                            let mut env = self.env.lock().unwrap();
                            env.define("G_TOKEN_REMAINING".to_string(), RuntimeValue::String(rest), true);
                        }
                    }
                    "QSORT" | "RESIZE" | "SETCLEAR" => {
                        if let Some(new_arr) = self.builtins.last_modified_array.take() {
                            if !args.is_empty() {
                                if let Expr::Variable(vname, vid) = &args[0] {
                                    self.perform_assignment(vname, *vid, RuntimeValue::Array(new_arr)).ok();
                                }
                            }
                        }
                    }
                    "GETDIR" => {
                        let files = self.builtins.getdir_files.drain(..).map(RuntimeValue::String).collect();
                        self.globals.lock().unwrap().define("GETDIR_FILES".into(), RuntimeValue::Array(files), false);
                    }
                    _ => {}
                }

                Ok(res)
            }
        }
    }

    pub(crate) fn lookup_variable(&mut self, name: &str, id: usize) -> RuntimeValue {
        let name_up = name.to_uppercase();
        
        // Dynamic Special Variables (§8.1)
        match name_up.as_str() {
            "G_SCREEN_W" => {
                unsafe { return RuntimeValue::Integer(GetSystemMetrics(SM_CXSCREEN) as i64); }
            }
            "G_SCREEN_H" => {
                unsafe { return RuntimeValue::Integer(GetSystemMetrics(SM_CYSCREEN) as i64); }
            }
            "G_MOUSE_X" | "G_MOUSE_Y" => {
                if let Some(with_obj) = self.with_stack.last() {
                    match with_obj {
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if *case_care { name.to_string() } else { name.to_uppercase() };
                            if let Some(val) = map.get(&key) {
                                return val.clone();
                            }
                        }
                        _ => {}
                    }
                }
                return RuntimeValue::Empty;
            }
            "TRY_ERRMSG" => {
                return RuntimeValue::String(self.caught_error_msg.clone());
            }
            "TRY_ERRLINE" => {
                return RuntimeValue::Integer(self.caught_error_line as i64);
            }
            _ => {}
        }

        if let Some(&distance) = self.locals.get(&id) {
            if let Some(val) = Environment::get_at(self.env.clone(), distance, &name_up) {
                return val;
            }
        }
        if let Some(val) = self.globals.lock().unwrap().get(&name_up) {
            return val;
        }

        // WITH context check (§50)
        for obj in self.with_stack.iter().rev() {
            match obj {
                RuntimeValue::Hash { map, case_care } => {
                    let key = if *case_care { name.to_string() } else { name.to_uppercase() };
                    if let Some(val) = map.get(&key) {
                        return val.clone();
                    }
                }
                _ => {}
            }
        }

        RuntimeValue::Empty
    }

    fn call_function_or_builtin(&mut self, name: &str, eval_args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
        let name_up = name.to_uppercase();
        if let Some((params, body)) = self.functions.get(&name_up).cloned() {
            // Determine parent environment:
            // If it's a module call "MOD.FUNC", the parent should be the module's environment.
            let parent_env = if name_up.contains('.') {
                let parts: Vec<&str> = name_up.split('.').collect();
                self.modules.get(parts[0]).cloned().unwrap_or(self.globals.clone())
            } else {
                self.globals.clone()
            };

            let local_env_arc = std::sync::Arc::new(std::sync::Mutex::new(Environment::new_enclosed(parent_env)));
                    
            // Setup parameters
            {
                let mut local_env = local_env_arc.lock().unwrap();
                for (i, param) in params.iter().enumerate() {
                    let arg_val = if i < eval_args.len() {
                        eval_args[i].clone()
                    } else if let Some(ref def) = param.default_value {
                        self.eval_expr(def)?
                    } else {
                        RuntimeValue::Empty
                    };
                    local_env.define(param.name.to_uppercase(), arg_val, false);
                }
            }
            
            let mut call_eval = Evaluator::new(local_env_arc.clone(), self.globals.clone(), self.locals.clone(), self.ai.clone());
            call_eval.functions = self.functions.clone();
            call_eval.modules = self.modules.clone();
            
            let call_res = match call_eval.eval_stmts(&body) {
                Ok(ControlFlow::Return(v)) => Ok(v),
                Ok(ControlFlow::Exit) | Ok(ControlFlow::None) => Ok(RuntimeValue::Empty),
                Ok(ControlFlow::ExitExit(_)) => return Err("EXITEXIT".into()),
                Ok(cf) => Err(format!("Unexpected control flow in function: {:?}", cf)),
                Err(e) => Err(e),
            };

            // SYNC VAR Parameters (simplified for now, needs access to original Exprs if we move this)
            // Actually, call_function_or_builtin doesn't have access to original args easily.
            // Let's keep it as is for now.

            call_res
        } else {
            // Check builtins
            self.builtins.call(name, eval_args)
        }
    }

    fn assign_to_expr(&mut self, target: &Expr, value: RuntimeValue, op: &AssignOp, _id: usize) -> Result<RuntimeValue, String> {
        match target {
            Expr::Variable(name, var_id) => {
                let mut target_val = if matches!(op, AssignOp::Assign) {
                    RuntimeValue::Empty
                } else {
                    self.lookup_variable(name, *var_id)
                };
                let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                self.perform_assignment(name, *var_id, new_val)
            }
            Expr::ArrayAccess(array_expr, indices, _) => {
                if let Expr::Variable(name, var_id) = &**array_expr {
                    let mut root = self.lookup_variable(name, *var_id);
                    let mut eval_indices = Vec::new();
                    for idx_expr in indices {
                        eval_indices.push(self.eval_expr(idx_expr)?);
                    }
                    
                    self.update_collection_element(&mut root, &eval_indices, value, op)?;
                    self.perform_assignment(name, *var_id, root)
                } else {
                    Err("Only assignments to collection variables are currently supported".into())
                }
            }
            Expr::GlobalVariable(name, _) => {
                let mut globals = self.globals.lock().unwrap();
                let name_up = name.to_uppercase();
                let mut target_val = if matches!(op, AssignOp::Assign) {
                    RuntimeValue::Empty
                } else {
                    globals.values.get(&name_up).cloned().unwrap_or(RuntimeValue::Empty)
                };
                let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                globals.values.insert(name_up, new_val.clone());
                Ok(new_val)
            }
            Expr::ThisMember(member, _) => {
                if let Some(with_obj) = self.with_stack.last_mut() {
                    match with_obj {
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if *case_care { member.clone() } else { member.to_uppercase() };
                            let mut target_val = if matches!(op, AssignOp::Assign) {
                                RuntimeValue::Empty
                            } else {
                                map.get(&key).cloned().unwrap_or(RuntimeValue::Empty)
                            };
                            let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                            map.insert(key, new_val.clone());
                            return Ok(new_val);
                        }
                        _ => {}
                    }
                }
                // Fallback to current env (lexical THIS)
                let name_up = member.to_uppercase();
                let mut env = self.env.lock().unwrap();
                let mut target_val = if matches!(op, AssignOp::Assign) {
                    RuntimeValue::Empty
                } else {
                    env.values.get(&name_up).cloned().unwrap_or(RuntimeValue::Empty)
                };
                let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                env.values.insert(name_up, new_val.clone());
                Ok(new_val)
            }
            Expr::WithMember(member, _) => {
                for with_obj in self.with_stack.iter_mut().rev() {
                    match with_obj {
                        RuntimeValue::Hash { map, case_care } => {
                            let key = if *case_care { member.clone() } else { member.to_uppercase() };
                            if matches!(op, AssignOp::Assign) || map.contains_key(&key) {
                                let mut target_val = if matches!(op, AssignOp::Assign) {
                                    RuntimeValue::Empty
                                } else {
                                    map.get(&key).cloned().unwrap_or(RuntimeValue::Empty)
                                };
                                let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                                map.insert(key, new_val.clone());
                                return Ok(new_val);
                            }
                        }
                        _ => {}
                    }
                }
                Err(format!("Member '{}' not found in WITH stack for assignment", member))
            }
            Expr::MemberAccess(expr, member, _) => {
                let mut left_val = self.eval_expr(expr)?;
                match &mut left_val {
                    RuntimeValue::Hash { map, case_care } => {
                        let key = if *case_care { member.clone() } else { member.to_uppercase() };
                        let mut target_val = if matches!(op, AssignOp::Assign) {
                            RuntimeValue::Empty
                        } else {
                            map.get(&key).cloned().unwrap_or(RuntimeValue::Empty)
                        };
                        let new_val = do_apply_assign_op(&mut target_val, value, op)?;
                        map.insert(key, new_val.clone());
                        
                        // We need to write back the modified hash to its original variable if it's a variable
                        if let Expr::Variable(name, var_id) = &**expr {
                            self.perform_assignment(name, *var_id, left_val)?;
                        } else if let Expr::GlobalVariable(name, _) = &**expr {
                             self.globals.lock().unwrap().assign(name, left_val.clone())?;
                        }
                        
                        Ok(new_val)
                    }
                    _ => Err(format!("Cannot assign to member of type {}", left_val.type_name()))
                }
            }
            _ => Err(format!("Invalid assignment target: {:?}", target)),
        }
    }
}

impl Evaluator {

    fn perform_assignment(&mut self, name: &str, id: usize, value: RuntimeValue) -> Result<RuntimeValue, String> {
        let name_up = name.to_uppercase();
        if let Some(&distance) = self.locals.get(&id) {
            Environment::assign_at(self.env.clone(), distance, name_up, value.clone())?;
        } else {
            if !self.globals.lock().unwrap().assign(&name_up, value.clone())? {
                if self.explicit_declaration {
                    return Err(format!("Variable '{}' not declared (OPTION EXPLICIT is active)", name_up));
                }
                self.globals.lock().unwrap().define(name_up, value.clone(), false);
            }
        }
        Ok(value)
    }

    fn update_collection_element(&mut self, coll_val: &mut RuntimeValue, indices: &[RuntimeValue], value: RuntimeValue, op: &AssignOp) -> Result<(), String> {
        if indices.is_empty() {
            *coll_val = do_apply_assign_op(coll_val, value, op)?;
            return Ok(());
        }

        let idx_val = &indices[0];
        match coll_val {
            RuntimeValue::Array(vals) => {
                let idx = idx_val.as_i64() as usize;
                if idx < vals.len() {
                    self.update_collection_element(&mut vals[idx], &indices[1..], value, op)
                } else {
                    Err(format!("Array index out of bounds: {}", idx))
                }
            }
            RuntimeValue::Hash { map, case_care } => {
                let key = if *case_care { idx_val.to_string() } else { idx_val.to_string().to_uppercase() };
                if !map.contains_key(&key) {
                    map.insert(key.clone(), RuntimeValue::Empty);
                }
                self.update_collection_element(map.get_mut(&key).unwrap(), &indices[1..], value, op)
            }
            _ => Err("Cannot index non-collection value during assignment".into()),
        }
    }
}
