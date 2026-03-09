use super::{Parser, Stmt, Expr};

impl Parser {
    /// 実験的な文法のパース試行
    #[allow(dead_code)]
    pub(crate) fn try_parse_experimental_stmt(&mut self) -> Option<Stmt> {
        // match self.peek() {
        //     Token::At => Some(self.at_statement()),
        //     _ => None,
        // }
        None
    }

    /// 実験的な式のパース試行
    #[allow(dead_code)]
    pub(crate) fn try_parse_experimental_expr(&mut self) -> Option<Expr> {
        None
    }
}
