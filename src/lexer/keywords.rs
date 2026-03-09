use super::Lexer;
use super::token::Token;

impl<'a> Lexer<'a> {
    pub(super) fn read_identifier(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        match s.to_uppercase().as_str() {
            "IF" => Token::If,
            "IFB" => Token::IfB,
            "THEN" => Token::Then,
            "ELSE" => Token::Else,
            "ELSEIF" => Token::ElseIf,
            "ENDIF" => Token::EndIf,
            "FOR" => Token::For,
            "TO" => Token::To,
            "STEP" => Token::Step,
            "IN" => Token::In,
            "NEXT" => Token::Next,
            "WHILE" => Token::While,
            "WEND" => Token::Wend,
            "REPEAT" => Token::Repeat,
            "UNTIL" => Token::Until,
            "SELECT" => Token::Select,
            "CASE" => Token::Case,
            "DEFAULT" => Token::Default,
            "SELEND" => Token::SelEnd,
            "PROCEDURE" => Token::Procedure,
            "FUNCTION" => Token::Function,
            "FEND" => Token::Fend,
            "MODULE" => Token::Module,
            "ENDMODULE" => Token::EndModule,
            "CLASS" => Token::Class,
            "ENDCLASS" => Token::EndClass,
            "DIM" => Token::Dim,
            "PUBLIC" => Token::Public,
            "CONST" => Token::Const,
            "HASHTBL" => Token::Hashtbl,
            "TRY" => Token::Try,
            "FINALLY" => Token::Finally,
            "EXCEPT" => Token::Except,
            "ENDTRY" => Token::EndTry,
            "WITH" => Token::With,
            "ENDWITH" => Token::EndWith,
            "ENUM" => Token::Enum,
            "ENDENUM" => Token::EndEnum,
            "TEXTBLOCK" => {
                self.skip_whitespace();
                let mut name = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        name.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                // Skip the rest of the current line
                while let Some(ch) = self.peek() {
                    if ch == '\n' { 
                        self.advance(); 
                        break; 
                    }
                    self.advance();
                }

                let mut content = String::new();
                let mut current_line = String::new();
                
                loop {
                    match self.advance() {
                        Some('\n') => {
                            if current_line.trim().to_uppercase() == "ENDTEXTBLOCK" {
                                break;
                            }
                            content.push_str(&current_line);
                            content.push('\n');
                            current_line.clear();
                        }
                        Some('\r') => {
                            if self.peek() == Some('\n') {
                                self.advance();
                            }
                            if current_line.trim().to_uppercase() == "ENDTEXTBLOCK" {
                                break;
                            }
                            content.push_str(&current_line);
                            content.push('\n');
                            current_line.clear();
                        }
                        Some(c) => current_line.push(c),
                        None => {
                            if current_line.trim().to_uppercase() != "ENDTEXTBLOCK" {
                                content.push_str(&current_line);
                            }
                            break;
                        }
                    }
                }

                self.buffer.push_back((Token::Identifier(name), self.line));
                self.buffer.push_back((Token::String(content), self.line));
                self.buffer.push_back((Token::EndTextBlock, self.line));
                Token::TextBlock
            }
            "ENDTEXTBLOCK" => Token::EndTextBlock,
            "CALL" => Token::Call,
            "BREAK" => Token::Break,
            "CONTINUE" => Token::Continue,
            "EXIT" => Token::Exit,
            "EXITEXIT" => Token::ExitExit,
            "PRINT" => Token::Print,
            "OPTION" => Token::Option,
            "DEF_DLL" => Token::DefDll,
            "THREAD" => Token::Thread,
            "RESULT" => Token::Result,
            "VAR" => Token::Var,
            "TRUE" => Token::Boolean(true),
            "FALSE" => Token::Boolean(false),
            "NULL" => Token::Null,
            "EMPTY" => Token::Empty,
            "NOTHING" => Token::Nothing,
            "MOD" => {
                if self.peek() == Some('=') {
                    self.advance(); // consume '='
                    Token::ModAssign
                } else {
                    Token::Mod
                }
            }
            "AND" => Token::And,
            "OR" => Token::Or,
            "XOR" => Token::Xor,
            "GLOBAL" => Token::Global,
            "THIS" => Token::This,
            _ => Token::Identifier(s),
        }
    }
}
