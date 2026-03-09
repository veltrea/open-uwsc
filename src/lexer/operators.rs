use super::Lexer;
use super::token::Token;

impl<'a> Lexer<'a> {
    pub(super) fn read_operator(&mut self, ch: char) -> Token {
        match ch {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '.' => Token::Dot,
            '_' => {
                // Check for line continuation: _ followed by optional whitespace and a newline
                self.skip_whitespace();
                match self.peek() {
                    Some('\n') | Some('\r') => {
                        if self.peek() == Some('\r') {
                            self.advance();
                            if self.peek() == Some('\n') {
                                self.advance();
                            }
                        } else {
                            self.advance();
                        }
                        return Token::Ref;
                    }
                    _ => Token::Underscore,
                }
            }
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::AddAssign
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::SubAssign
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::AddAssign // FIXME: Should be MulAssign, but matching existing code for now
                } else {
                    Token::Asterisk
                }
            }
            '/' => {
                if self.peek() == Some('/') {
                    // Comment until end of line
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        self.advance();
                    }
                    Token::Ref
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::DivAssign
                } else {
                    Token::Slash
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::Equal
                } else {
                    Token::Assign
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GreaterEq
                } else {
                    Token::Greater
                }
            }
            '<' => {
                match self.peek() {
                    Some('=') => {
                        self.advance();
                        Token::LessEq
                    }
                    Some('>') => {
                        self.advance();
                        Token::NotEqual
                    }
                    Some('#') => {
                        // Special Constant <#...>
                        self.advance(); // consume #
                        let mut name = String::new();
                        while let Some(c) = self.peek() {
                            if c == '>' {
                                self.advance(); // consume >
                                break;
                            }
                            name.push(self.advance().unwrap());
                        }
                        Token::SpecialConst(name)
                    }
                    _ => Token::Less
                }
            }
            '!' => Token::Not,
            _ => Token::Identifier(ch.to_string()),
        }
    }
}
