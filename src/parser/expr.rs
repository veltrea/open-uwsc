use super::{Parser, Expr, Literal, BinaryOp, UnaryOp, AssignOp};
use crate::lexer::Token;

impl Parser {
    pub(crate) fn expression(&mut self) -> Expr {
        #[cfg(feature = "experimental")]
        if let Some(expr) = self.try_parse_experimental_expr() {
            return expr;
        }
        // println!("DEBUG: expression() at {:?}", self.peek());
        self.assignment()
    }

    pub(crate) fn assignment(&mut self) -> Expr {
        // println!("DEBUG: assignment() at {:?}", self.peek());
        let expr = self.logic_or();

        let op = match self.peek() {
            Token::Assign => Some(AssignOp::Assign),
            Token::AddAssign => Some(AssignOp::AddAssign),
            Token::SubAssign => Some(AssignOp::SubAssign),
            Token::MulAssign => Some(AssignOp::MulAssign),
            Token::DivAssign => Some(AssignOp::DivAssign),
            Token::ModAssign => Some(AssignOp::ModAssign),
            _ => None,
        };

        if let Some(assign_op) = op {
            self.advance(); // consume op
            let value = self.assignment();
            if matches!(expr, Expr::Variable(_, _) | Expr::ArrayAccess(_, _, _) | Expr::MemberAccess(_, _, _) | Expr::WithMember(_, _) | Expr::GlobalVariable(_, _) | Expr::ThisMember(_, _)) {
                let id = self.get_id();
                return Expr::Assign(Box::new(expr), assign_op, Box::new(value), id);
            }
            panic!("Invalid assignment target");
        }

        expr
    }

    pub(crate) fn logic_or(&mut self) -> Expr {
        let mut expr = self.logic_and();
        while self.match_token(Token::Or) || self.match_token(Token::Xor) {
            let op = if self.previous() == Token::Or { BinaryOp::Or } else { BinaryOp::Xor };
            let right = self.logic_and();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    pub(crate) fn logic_and(&mut self) -> Expr {
        let mut expr = self.equality();
        while self.match_token(Token::And) {
            let right = self.equality();
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }
        expr
    }

    pub(crate) fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.match_token(Token::NotEqual) || self.match_token(Token::Equal) {
            let op = match self.previous() {
                Token::NotEqual => BinaryOp::NotEqual,
                _ => BinaryOp::Equal,
            };
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    pub(crate) fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.match_token(Token::Greater) || self.match_token(Token::GreaterEq) || self.match_token(Token::Less) || self.match_token(Token::LessEq) {
            let op = match self.previous() {
                Token::Greater => BinaryOp::Greater,
                Token::GreaterEq => BinaryOp::GreaterEq,
                Token::Less => BinaryOp::Less,
                Token::LessEq => BinaryOp::LessEq,
                _ => unreachable!(),
            };
            let right = self.term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    pub(crate) fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.check(Token::Minus) || self.check(Token::Plus) {
            let op_token = self.advance();
            let op = if op_token == Token::Minus { BinaryOp::Sub } else { BinaryOp::Plus };
            let right = self.factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    pub(crate) fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.check(Token::Slash) || self.check(Token::Asterisk) || self.check(Token::Mod) {
            let op_token = self.advance();
            let op = match op_token {
                Token::Slash => BinaryOp::Div,
                Token::Asterisk => BinaryOp::Mul,
                Token::Mod => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    pub(crate) fn unary(&mut self) -> Expr {
        if self.match_token(Token::Not) || self.match_token(Token::Minus) {
            let op = if self.previous() == Token::Not { UnaryOp::Not } else { UnaryOp::Neg };
            let right = self.unary();
            return Expr::Unary(op, Box::new(right));
        }
        self.call()
    }

    pub(crate) fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        loop {
            if self.match_token(Token::LParen) {
                expr = self.finish_call(expr);
            } else if self.match_token(Token::LBracket) {
                expr = self.finish_array_access(expr);
            } else if self.match_token(Token::Dot) {
                let member = if let Token::Identifier(m) = self.advance() { m } else { panic!("Expected member name after '.'") };
                let id = self.get_id();
                expr = Expr::MemberAccess(Box::new(expr), member, id);
            } else {
                break;
            }
        }
        expr
    }

    pub(crate) fn finish_array_access(&mut self, array: Expr) -> Expr {
        let mut indices = Vec::new();
        loop {
            indices.push(self.expression());
            if !self.match_token(Token::Comma) { break; }
        }
        self.consume(Token::RBracket, "Expected ']' after array index");
        let id = self.get_id();
        Expr::ArrayAccess(Box::new(array), indices, id)
    }

    pub(crate) fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut args = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                args.push(self.expression());
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RParen, "Expected ')' after arguments");
        let id = self.get_id();
        Expr::Call(Box::new(callee), args, id)
    }

    pub(crate) fn primary(&mut self) -> Expr {
        // println!("DEBUG: primary() at {:?}", self.peek());
        let token = self.advance();
        match token {
            Token::Boolean(b) => Expr::Literal(Literal::Boolean(b)),
            Token::SpecialConst(s) => Expr::Literal(Literal::SpecialConst(s)),
            Token::Null => Expr::Literal(Literal::Null),
            Token::Empty => Expr::Literal(Literal::Empty),
            Token::Number(n) => Expr::Literal(Literal::Number(n)),
            Token::String(s) => Expr::Literal(Literal::String(s)),
            Token::Identifier(s) => {
                let id = self.get_id();
                Expr::Variable(s, id)
            }
            Token::LParen => {
                let expr = self.expression();
                self.consume(Token::RParen, "Expected ')' after expression");
                expr
            }
            Token::LBracket => {
                let mut elements = Vec::new();
                if !self.check(Token::RBracket) {
                    loop {
                        elements.push(self.expression());
                        if !self.match_token(Token::Comma) { break; }
                    }
                }
                self.consume(Token::RBracket, "Expected ']' after array elements");
                let id = self.get_id();
                Expr::ArrayLiteral(elements, id)
            }
            Token::Dot => {
                let id = self.get_id();
                let member = if let Token::Identifier(name) = self.peek() {
                    self.advance();
                    name
                } else {
                    return Expr::Literal(Literal::Null); // Error or Null
                };
                Expr::WithMember(member, id)
            }
            Token::Global => {
                self.consume(Token::Dot, "Expected '.' after GLOBAL");
                let member = if let Token::Identifier(name) = self.advance() {
                    name
                } else {
                    panic!("Expected identifier after GLOBAL.");
                };
                let id = self.get_id();
                Expr::GlobalVariable(member, id)
            }
            Token::This => {
                self.consume(Token::Dot, "Expected '.' after THIS");
                let member = if let Token::Identifier(name) = self.advance() {
                    name
                } else {
                    panic!("Expected identifier after THIS.");
                };
                let id = self.get_id();
                Expr::ThisMember(member, id)
            }
            _ => {
                panic!("Expected expression, found {:?}", token);
            }
        }
    }
}
