use super::{Parser, Stmt, Token, Parameter, Expr};

impl Parser {
    pub(crate) fn declaration(&mut self) -> Option<Stmt> {
        self.skip_newlines();
        if self.is_at_end() { return None; }

        match self.peek() {
            Token::Wend | Token::EndIf | Token::Next | Token::Until | Token::Fend | Token::SelEnd | Token::Case | Token::Default | Token::Else | Token::ElseIf => {
                return None;
            }
            _ => {}
        }

        let stmt = match self.peek() {
            Token::Dim => self.dim_declaration(),
            Token::Public | Token::Global => self.public_declaration(),
            Token::Const => self.const_declaration(),
            Token::Procedure => self.procedure_declaration(),
            Token::Function => self.function_declaration(),
            Token::DefDll => self.def_dll_statement(),
            Token::Thread => self.thread_statement(),
            _ => self.statement(),
        };

        self.skip_newlines();
        Some(stmt)
    }

    pub(crate) fn dim_declaration(&mut self) -> Stmt {
        self.advance(); // DIM
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected identifier after DIM") };
        let mut dimensions = Vec::new();
        if self.match_token(Token::LBracket) {
            if !self.check(Token::RBracket) {
                loop {
                    dimensions.push(self.expression());
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RBracket, "Expected ']' after dimensions");
        }
        let mut init = None;
        if self.match_token(Token::Assign) {
            let mut list = Vec::new();
            loop {
                list.push(self.expression());
                if !self.match_token(Token::Comma) { break; }
            }
            if list.len() > 1 {
                let id = self.get_id();
                init = Some(Expr::ArrayLiteral(list, id));
            } else {
                init = Some(list.remove(0));
            }
        }
        let id = self.get_id();
        Stmt::Dim { name, init, dimensions, id }
    }

    pub(crate) fn public_declaration(&mut self) -> Stmt {
        self.advance(); // PUBLIC
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected identifier after PUBLIC") };
        let mut dimensions = Vec::new();
        if self.match_token(Token::LBracket) {
            if !self.check(Token::RBracket) {
                loop {
                    dimensions.push(self.expression());
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RBracket, "Expected ']' after dimensions");
        }
        let mut init = None;
        if self.match_token(Token::Assign) {
            let mut list = Vec::new();
            loop {
                list.push(self.expression());
                if !self.match_token(Token::Comma) { break; }
            }
            if list.len() > 1 {
                let id = self.get_id();
                init = Some(Expr::ArrayLiteral(list, id));
            } else {
                init = Some(list.remove(0));
            }
        }
        let id = self.get_id();
        Stmt::Public { name, init, dimensions, id }
    }

    pub(crate) fn const_declaration(&mut self) -> Stmt {
        self.advance(); // CONST
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected identifier after CONST") };
        self.consume(Token::Assign, "Expected '=' after CONST name (initialization is required)");
        let init = self.expression();
        let id = self.get_id();
        Stmt::Const(name, init, id)
    }

    pub(crate) fn procedure_declaration(&mut self) -> Stmt {
        self.advance(); // PROCEDURE
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected procedure name") };
        self.consume(Token::LParen, "Expected '(' after procedure name");
        let params = self.parse_parameters();
        self.consume(Token::RParen, "Expected ')' after parameters");
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(Token::Fend) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::Fend, "Expected FEND after procedure body");
        Stmt::Procedure { name, params, body }
    }

    pub(crate) fn function_declaration(&mut self) -> Stmt {
        self.advance(); // FUNCTION
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected function name") };
        self.consume(Token::LParen, "Expected '(' after function name");
        let params = self.parse_parameters();
        self.consume(Token::RParen, "Expected ')' after parameters");
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(Token::Fend) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::Fend, "Expected FEND after function body");
        Stmt::Function { name, params, body }
    }

    pub(crate) fn parse_parameters(&mut self) -> Vec<Parameter> {
        let mut params = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                let is_var = self.match_token(Token::Var) || self.match_token(Token::Ref); // Support Ref as alias
                let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected parameter name") };
                let is_array = if self.match_token(Token::LBracket) {
                    self.consume(Token::RBracket, "Expected ']' after array parameter indicator");
                    true
                } else {
                    false
                };
                let mut default_value = None;
                if self.match_token(Token::Assign) {
                    default_value = Some(self.expression());
                }
                params.push(Parameter { name, is_var, is_array, default_value });
                if !self.match_token(Token::Comma) { break; }
            }
        }
        params
    }

    pub(crate) fn def_dll_statement(&mut self) -> Stmt {
        self.advance(); // DEF_DLL
        
        let name = if let Token::Identifier(n) = self.advance() {
            n
        } else {
            panic!("Expected function name after DEF_DLL");
        };

        self.consume(Token::LParen, "Expected '(' after DEF_DLL function name");
        let params = self.parse_parameters();
        self.consume(Token::RParen, "Expected ')' after parameters");

        self.consume(Token::Colon, "Expected ':' before return type");
        let ret_type = if let Token::Identifier(t) = self.advance() {
            t
        } else {
            "void".to_string()
        };

        self.consume(Token::Colon, "Expected ':' before DLL path");
        let dll_path = match self.advance() {
            Token::String(s) => s,
            Token::Identifier(mut s) => {
                while self.match_token(Token::Dot) {
                    s.push('.');
                    if let Token::Identifier(next) = self.advance() {
                        s.push_str(&next);
                    }
                }
                s
            }
            _ => panic!("Expected DLL path after second colon"),
        };

        Stmt::DefDll { name, params, ret_type, dll_path }
    }

    pub(crate) fn thread_statement(&mut self) -> Stmt {
        self.advance(); // THREAD
        let call = self.expression();
        Stmt::Thread(call)
    }
}
