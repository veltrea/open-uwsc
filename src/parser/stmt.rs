use super::{Parser, Stmt, Token};

impl Parser {
    pub(crate) fn statement(&mut self) -> Stmt {
        self.skip_newlines();
        #[cfg(feature = "experimental")]
        if let Some(stmt) = self.try_parse_experimental_stmt() {
            return stmt;
        }

        match self.peek() {
            Token::If => self.single_if_statement(),
            Token::IfB => self.ifb_statement(),
            Token::While => self.while_statement(),
            Token::Repeat => self.repeat_statement(),
            Token::Select => self.select_statement(),
            Token::Try => self.try_statement(),
            Token::For => self.for_statement(),
            Token::Print => self.print_statement(),
            Token::Result => self.return_statement(),
            Token::Break => self.break_statement(),
            Token::Continue => self.continue_statement(),
            Token::Exit => self.exit_statement(),
            Token::ExitExit => self.exitexit_statement(),
            Token::Call => self.call_statement(),
            Token::Hashtbl => self.hashtbl_statement(),
            Token::With => self.with_statement(),
            Token::TextBlock => self.textblock_statement(),
            Token::Module => self.module_statement(),
            Token::Class => self.class_statement(),
            Token::Enum => self.enum_statement(),
            Token::Option => self.option_statement(),
            Token::Wend | Token::EndIf | Token::Next | Token::Until | Token::Fend | Token::SelEnd | Token::Case | Token::Default | Token::Else | Token::ElseIf => {
                panic!("Unexpected block terminator token: {:?}", self.peek());
            }
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn exit_statement(&mut self) -> Stmt {
        let id = self.get_id();
        self.advance(); // EXIT
        Stmt::Exit(id)
    }

    pub(crate) fn exitexit_statement(&mut self) -> Stmt {
        let id = self.get_id();
        self.advance(); // EXITEXIT
        let mut code = None;
        if !self.check(Token::Newline) && !self.check(Token::Semicolon) && !self.is_at_end() {
            // Check if what follows can be an expression
            match self.peek() {
                Token::Number(_) | Token::String(_) | Token::Identifier(_) | Token::LParen | Token::Minus | Token::Not | Token::Boolean(_) => {
                    code = Some(self.expression());
                }
                _ => {}
            }
        }
        Stmt::ExitExit(code, id)
    }

    pub(crate) fn call_statement(&mut self) -> Stmt {
        let id = self.get_id();
        self.advance(); // CALL
        let path = self.expression();
        Stmt::Call(path, id)
    }

    pub(crate) fn break_statement(&mut self) -> Stmt {
        self.advance(); // BREAK
        let mut depth = None;
        if let Token::Number(n) = self.peek() {
            self.advance();
            depth = Some(n as usize);
        }
        Stmt::Break(depth)
    }

    pub(crate) fn continue_statement(&mut self) -> Stmt {
        self.advance(); // CONTINUE
        let mut depth = None;
        if let Token::Number(n) = self.peek() {
            self.advance();
            depth = Some(n as usize);
        }
        Stmt::Continue(depth)
    }

    pub(crate) fn hashtbl_statement(&mut self) -> Stmt {
        let id = self.get_id();
        self.advance(); // HASHTBL
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name for HASHTBL") };
        let mut init = None;
        if self.match_token(Token::Assign) {
            init = Some(self.expression());
        }
        Stmt::Hashtbl(name, init, id)
    }

    pub(crate) fn textblock_statement(&mut self) -> Stmt {
        let id = self.get_id();
        self.advance(); // TEXTBLOCK
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name for TEXTBLOCK") };
        let content = if let Token::String(s) = self.advance() { s } else { panic!("Expected content for TEXTBLOCK") };
        self.consume(Token::EndTextBlock, "Expected ENDTEXTBLOCK");
        Stmt::TextBlock { name, content, id }
    }

    pub(crate) fn option_statement(&mut self) -> Stmt {
        self.advance(); // OPTION
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name after OPTION") };
        let id = self.get_id();
        Stmt::Option(name, id)
    }

    pub(crate) fn ifb_statement(&mut self) -> Stmt {
        self.advance(); // IFB
        let condition = self.expression();
        self.match_token(Token::Then); // THEN is optional in IFB block
        let id = self.get_id();
        self.skip_newlines();

        let mut then_branch = Vec::new();
        while !self.check(Token::Else) && !self.check(Token::ElseIf) && !self.check(Token::EndIf) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                then_branch.push(s);
            }
        }

        let mut else_if_blocks = Vec::new();
        while self.match_token(Token::ElseIf) {
            let elif_condition = self.expression();
            self.match_token(Token::Then);
            self.skip_newlines();
            let mut elif_stmts = Vec::new();
            while !self.check(Token::Else) && !self.check(Token::ElseIf) && !self.check(Token::EndIf) && !self.is_at_end() {
                self.skip_newlines();
                if self.check(Token::Else) || self.check(Token::ElseIf) || self.check(Token::EndIf) { break; }
                if let Some(s) = self.declaration() {
                    elif_stmts.push(s);
                }
            }
            else_if_blocks.push((elif_condition, elif_stmts));
        }

        let mut else_branch = None;
        if self.match_token(Token::Else) {
            self.skip_newlines();
            let mut else_stmts = Vec::new();
            while !self.check(Token::EndIf) && !self.is_at_end() {
                self.skip_newlines();
                if self.check(Token::EndIf) { break; }
                if let Some(s) = self.declaration() {
                    else_stmts.push(s);
                }
            }
            else_branch = Some(else_stmts);
        }

        self.consume(Token::EndIf, "Expected ENDIF");
        Stmt::If { condition, then_branch, else_if_blocks, else_branch, id }
    }

    pub(crate) fn single_if_statement(&mut self) -> Stmt {
        self.advance(); // IF
        let condition = self.expression();
        self.consume(Token::Then, "Expected THEN after IF");
        let then_stmt = self.statement();
        let mut else_branch = None;
        if self.match_token(Token::Else) {
            else_branch = Some(vec![self.statement()]);
        }
        let id = self.get_id();
        Stmt::If { condition, then_branch: vec![then_stmt], else_if_blocks: vec![], else_branch, id }
    }

    pub(crate) fn with_statement(&mut self) -> Stmt {
        self.advance(); // WITH
        let expr = self.expression();
        let id = self.get_id();
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(Token::EndWith) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(Token::EndWith) { break; }
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::EndWith, "Expected ENDWITH");
        Stmt::With { expr, body, id }
    }

    pub(crate) fn while_statement(&mut self) -> Stmt {
        self.advance(); // WHILE
        let condition = self.expression();
        let id = self.get_id();
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(Token::Wend) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(Token::Wend) { break; }
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::Wend, "Expected WEND");
        Stmt::While { condition, body, id }
    }

    pub(crate) fn repeat_statement(&mut self) -> Stmt {
        self.advance(); // REPEAT
        let id = self.get_id();
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(Token::Until) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(Token::Until) { break; }
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }

        self.consume(Token::Until, "Expected UNTIL after REPEAT body");
        let condition = self.expression();
        Stmt::Repeat { body, condition, id }
    }

    pub(crate) fn select_statement(&mut self) -> Stmt {
        self.advance(); // SELECT
        let target = self.expression();
        let id = self.get_id();
        self.skip_newlines();

        let mut cases = Vec::new();
        let mut default_branch = None;

        while !self.check(Token::SelEnd) && !self.is_at_end() {
            if self.match_token(Token::Case) {
                let mut case_exprs = Vec::new();
                loop {
                    case_exprs.push(self.expression());
                    if !self.match_token(Token::Comma) { break; }
                }
                self.skip_newlines();
                let mut body = Vec::new();
                while !self.check(Token::Case) && !self.check(Token::Default) && !self.check(Token::SelEnd) && !self.is_at_end() {
                    self.skip_newlines();
                    if self.check(Token::Case) || self.check(Token::Default) || self.check(Token::SelEnd) { break; }
                    if let Some(s) = self.declaration() {
                        body.push(s);
                    }
                }
                cases.push((case_exprs, body));
            } else if self.match_token(Token::Default) {
                self.skip_newlines();
                let mut body = Vec::new();
                while !self.check(Token::SelEnd) && !self.is_at_end() {
                    self.skip_newlines();
                    if self.check(Token::SelEnd) { break; }
                    if let Some(s) = self.declaration() {
                        body.push(s);
                    }
                }
                default_branch = Some(body);
                break;
            } else {
                self.advance();
            }
        }

        self.consume(Token::SelEnd, "Expected SELEND");
        Stmt::Select { target, cases, default_branch, id }
    }

    pub(crate) fn try_statement(&mut self) -> Stmt {
        self.advance(); // TRY
        let id = self.get_id();
        self.skip_newlines();

        let mut try_branch = Vec::new();
        while !self.check(Token::Except) && !self.check(Token::Finally) && !self.check(Token::EndTry) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                try_branch.push(s);
            }
        }

        let mut except_branch = None;
        if self.match_token(Token::Except) {
            self.skip_newlines();
            let mut except_stmts = Vec::new();
            while !self.check(Token::Finally) && !self.check(Token::EndTry) && !self.is_at_end() {
                if let Some(s) = self.declaration() {
                    except_stmts.push(s);
                }
            }
            except_branch = Some(except_stmts);
        }

        let mut finally_branch = None;
        if self.match_token(Token::Finally) {
            self.skip_newlines();
            let mut finally_stmts = Vec::new();
            while !self.check(Token::EndTry) && !self.is_at_end() {
                if let Some(s) = self.declaration() {
                    finally_stmts.push(s);
                }
            }
            finally_branch = Some(finally_stmts);
        }

        self.consume(Token::EndTry, "Expected ENDTRY");
        Stmt::Try { try_branch, except_branch, finally_branch, id }
    }

    pub(crate) fn for_statement(&mut self) -> Stmt {
        self.advance(); // FOR
        let var = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected identifier in FOR") };
        
        if self.match_token(Token::In) {
            let array = self.expression();
            let id = self.get_id();
            self.skip_newlines();
            let mut body = Vec::new();
            while !self.check(Token::Next) && !self.is_at_end() {
                self.skip_newlines();
                if self.check(Token::Next) { break; }
                if let Some(s) = self.declaration() {
                    body.push(s);
                }
            }
            self.consume(Token::Next, "Expected NEXT");
            return Stmt::ForIn { var, array, body, id };
        }

        self.consume(Token::Assign, "Expected '=' or 'IN' in FOR");
        let from = self.expression();
        self.consume(Token::To, "Expected TO in FOR");
        let to = self.expression();
        let mut step = None;
        if self.match_token(Token::Step) {
            step = Some(self.expression());
        }
        let id = self.get_id();
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(Token::Next) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::Next, "Expected NEXT");
        Stmt::For { var, from, to, step, body, id }
    }

    pub(crate) fn print_statement(&mut self) -> Stmt {
        self.advance(); // PRINT
        let expr = self.expression();
        let id = self.get_id();
        Stmt::Print(expr, id)
    }

    pub(crate) fn return_statement(&mut self) -> Stmt {
        self.advance(); // RESULT
        self.consume(Token::Assign, "Expected '=' after RESULT");
        let expr = self.expression();
        Stmt::Return(Some(expr))
    }

    pub(crate) fn module_statement(&mut self) -> Stmt {
        self.advance(); // MODULE
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name for MODULE") };
        let id = self.get_id();
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(Token::EndModule) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::EndModule, "Expected ENDMODULE");
        Stmt::Module { name, body, id }
    }

    pub(crate) fn class_statement(&mut self) -> Stmt {
        self.advance(); // CLASS
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name for CLASS") };
        let id = self.get_id();
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(Token::EndClass) && !self.is_at_end() {
            if let Some(s) = self.declaration() {
                body.push(s);
            }
        }
        self.consume(Token::EndClass, "Expected ENDCLASS");
        // For now, treat CLASS like MODULE (singleton-like) in the AST
        Stmt::Module { name, body, id }
    }

    pub(crate) fn enum_statement(&mut self) -> Stmt {
        self.advance(); // ENUM
        let name = if let Token::Identifier(n) = self.advance() { n } else { panic!("Expected name for ENUM") };
        let id = self.get_id();
        self.skip_newlines();
        let mut members = Vec::new();
        while !self.check(Token::EndEnum) && !self.is_at_end() {
            if let Token::Identifier(mname) = self.advance() {
                let mut val = None;
                if self.match_token(Token::Assign) {
                    val = Some(self.expression());
                }
                members.push((mname, val));
            }
            self.match_token(Token::Comma);
            self.skip_newlines();
        }
        self.consume(Token::EndEnum, "Expected ENDENUM");
        Stmt::Enum { name, members, id }
    }

    pub(crate) fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        let id = self.get_id();
        self.skip_newlines();
        Stmt::Expression(expr, id)
    }
}
