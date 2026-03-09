use crate::lexer::Token;
use std::collections::HashMap;

pub mod expr;
pub mod stmt;
pub mod decl;
#[cfg(feature = "experimental")]
pub mod experimental;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String, usize),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>, usize),
    Assign(Box<Expr>, AssignOp, Box<Expr>, usize),
    ArrayLiteral(Vec<Expr>, usize),
    ArrayAccess(Box<Expr>, Vec<Expr>, usize),
    MemberAccess(Box<Expr>, String, usize),
    WithMember(String, usize),
    GlobalVariable(String, usize),
    ThisMember(String, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Empty,
    SpecialConst(String),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual, Greater, Less, GreaterEq, LessEq,
    And, Or, Xor, Plus, Minus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOp {
    Eq, Ne, Gt, Lt, Ge, Le
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not, Neg,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub is_var: bool,
    pub is_array: bool,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr, usize),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_if_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
        id: usize,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        id: usize,
    },
    Repeat {
        body: Vec<Stmt>,
        condition: Expr,
        id: usize,
    },
    Select {
        target: Expr,
        cases: Vec<(Vec<Expr>, Vec<Stmt>)>,
        default_branch: Option<Vec<Stmt>>,
        id: usize,
    },
    Try {
        try_branch: Vec<Stmt>,
        except_branch: Option<Vec<Stmt>>,
        finally_branch: Option<Vec<Stmt>>,
        id: usize,
    },
    For {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
        id: usize,
    },
    ForIn {
        var: String,
        array: Expr,
        body: Vec<Stmt>,
        id: usize,
    },
    Print(Expr, usize),
    Procedure {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Stmt>,
    },
    Dim { name: String, init: Option<Expr>, dimensions: Vec<Expr>, id: usize },
    Public { name: String, init: Option<Expr>, dimensions: Vec<Expr>, id: usize },
    Exit(usize),
    ExitExit(Option<Expr>, usize),
    Call(Expr, usize),
    Hashtbl(String, Option<Expr>, usize),
    TextBlock { name: String, content: String, id: usize },
    Const(String, Expr, usize),
    Return(Option<Expr>),
    Module { name: String, body: Vec<Stmt>, id: usize },
    Enum { name: String, members: Vec<(String, Option<Expr>)>, id: usize },
    Break(Option<usize>),
    Continue(Option<usize>),
    Option(String, usize),
    DefDll {
        name: String,
        params: Vec<Parameter>, // Using Parameter for DLLs too, though is_var/default is special
        ret_type: String,
        dll_path: String,
    },
    Thread(Expr),
    With { expr: Expr, body: Vec<Stmt>, id: usize },
}

pub struct Parser {
    pub tokens: Vec<(Token, usize)>,
    pub(crate) current: usize,
    pub(crate) next_id: usize,
    pub(crate) current_line: usize,
    pub stmt_lines: HashMap<usize, usize>,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, usize)>) -> Self {
        let first_line = tokens.first().map(|(_, l)| *l).unwrap_or(1);
        Self { tokens, current: 0, next_id: 0, current_line: first_line, stmt_lines: HashMap::new() }
    }

    pub(crate) fn get_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.stmt_lines.insert(id, self.current_line);
        id
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            let start = self.current;
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
            if self.current == start && !self.is_at_end() {
                self.advance(); 
            }
        }
        stmts
    }

    pub(crate) fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }
        false
    }

    pub(crate) fn check(&self, token: Token) -> bool {
        if self.is_at_end() { return false; }
        self.peek() == token
    }

    pub(crate) fn advance(&mut self) -> Token {
        if !self.is_at_end() { 
            let (tok, line) = self.tokens[self.current].clone();
            self.current += 1; 
            self.current_line = line;
            tok
        } else {
            self.previous()
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].0 == Token::EOF
    }

    pub(crate) fn peek(&self) -> Token {
        if self.current >= self.tokens.len() {
             Token::EOF
        } else {
             self.tokens[self.current].0.clone()
        }
    }
 
    pub(crate) fn previous(&self) -> Token {
        if self.current == 0 {
            Token::EOF
        } else {
            self.tokens[self.current - 1].0.clone()
        }
    }

    pub(crate) fn consume(&mut self, token: Token, msg: &str) -> Token {
        if self.check(token) { return self.advance(); }
        panic!("{}", msg);
    }

    pub(crate) fn skip_newlines(&mut self) {
        while self.match_token(Token::Newline) || self.match_token(Token::Semicolon) {}
    }
}
