use crate::parser::{Stmt, Expr, Parameter};
use std::collections::HashMap;

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<usize, usize>, // ID -> distance
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, stmts: &[Stmt]) -> HashMap<usize, usize> {
        self.resolve_stmts(stmts);
        self.locals.clone()
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr, _) | Stmt::Print(expr, _) | Stmt::Thread(expr) | Stmt::Call(expr, _) => {
                self.resolve_expr(expr);
            }
            Stmt::If { condition, then_branch, else_if_blocks, else_branch, .. } => {
                self.resolve_expr(condition);
                self.resolve_stmts(then_branch);
                for (cond, branch) in else_if_blocks {
                    self.resolve_expr(cond);
                    self.resolve_stmts(branch);
                }
                if let Some(branch) = else_branch {
                    self.resolve_stmts(branch);
                }
            }
            Stmt::While { condition, body, .. } | Stmt::Repeat { body, condition, .. } => {
                self.resolve_expr(condition);
                self.resolve_stmts(body);
            }
            Stmt::Select { target, cases, default_branch, .. } => {
                self.resolve_expr(target);
                for (exprs, branch) in cases {
                    for e in exprs { self.resolve_expr(e); }
                    self.resolve_stmts(branch);
                }
                if let Some(branch) = default_branch {
                    self.resolve_stmts(branch);
                }
            }
            Stmt::Try { try_branch, except_branch, finally_branch, .. } => {
                self.resolve_stmts(try_branch);
                if let Some(branch) = except_branch { self.resolve_stmts(branch); }
                if let Some(branch) = finally_branch { self.resolve_stmts(branch); }
            }
            Stmt::For { var, from, to, step, body, .. } => {
                self.resolve_expr(from);
                self.resolve_expr(to);
                if let Some(s) = step { self.resolve_expr(s); }
                
                self.begin_scope();
                self.declare(var);
                self.define(var);
                self.resolve_stmts(body);
                self.end_scope();
            }
            Stmt::ForIn { var, array, body, .. } => {
                self.resolve_expr(array);
                self.begin_scope();
                self.declare(var);
                self.define(var);
                self.resolve_stmts(body);
                self.end_scope();
            }
            Stmt::Procedure { name, params, body } | Stmt::Function { name, params, body } => {
                // Functions are defined in the current scope (usually global)
                self.declare(name);
                self.define(name);
                self.resolve_function(params, body);
            }
            Stmt::Dim { name, init, dimensions, id } | Stmt::Public { name, init, dimensions, id } => {
                if let Some(expr) = init { self.resolve_expr(expr); }
                for d in dimensions { self.resolve_expr(d); }
                self.declare(name);
                self.define(name);
                self.resolve_local(name, *id);
            }
            Stmt::Const(name, expr, id) => {
                self.resolve_expr(expr);
                self.declare(name);
                self.define(name);
                self.resolve_local(name, *id);
            }
            Stmt::Module { body, .. } => {
                self.begin_scope();
                self.resolve_stmts(body);
                self.end_scope();
            }
            Stmt::With { expr, body, .. } => {
                self.resolve_expr(expr);
                self.resolve_stmts(body);
            }
            Stmt::Return(expr) | Stmt::ExitExit(expr, _) => {
                if let Some(e) = expr { self.resolve_expr(e); }
            }
            _ => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(name, id) => {
                self.resolve_local(name, *id);
            }
            Expr::Assign(target, _, value, id) => {
                self.resolve_expr(value);
                self.resolve_expr(target);
                if let Expr::Variable(name, _) = &**target {
                    self.resolve_local(name, *id);
                }
            }
            Expr::Binary(left, _, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary(_, expr) => {
                self.resolve_expr(expr);
            }
            Expr::Call(callee, args, id) => {
                self.resolve_expr(callee);
                for arg in args { self.resolve_expr(arg); }
                if let Expr::Variable(name, _) = &**callee {
                    self.resolve_local(name, *id);
                }
            }
            Expr::ArrayLiteral(elements, _) => {
                for e in elements { self.resolve_expr(e); }
            }
            Expr::ArrayAccess(array, indices, _) => {
                self.resolve_expr(array);
                for i in indices { self.resolve_expr(i); }
            }
            Expr::MemberAccess(expr, _, _) => {
                self.resolve_expr(expr);
            }
            _ => {}
        }
    }

    fn resolve_function(&mut self, params: &[Parameter], body: &[Stmt]) {
        self.begin_scope();
        for param in params {
            self.declare(&param.name);
            self.define(&param.name);
            if let Some(ref def) = param.default_value {
                self.resolve_expr(def);
            }
        }
        self.resolve_stmts(body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, name: &str, id: usize) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(name) {
                self.locals.insert(id, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}
