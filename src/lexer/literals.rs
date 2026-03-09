use super::Lexer;
use super::token::Token;

impl<'a> Lexer<'a> {
    pub(super) fn read_string(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' {
                break;
            }
            if ch == '<' && self.peek() == Some('#') {
                // Handle special characters like <#CR>, <#DBL>, <#TAB>
                let mut spec = String::new();
                self.advance(); // skip '#'
                while let Some(c) = self.peek() {
                    if c == '>' {
                        self.advance();
                        break;
                    }
                    spec.push(self.advance().unwrap());
                }
                match spec.as_str() {
                    "CR" => s.push('\n'),
                    "DBL" => s.push('"'),
                    "TAB" => s.push('\t'),
                    _ => {
                        s.push_str("<#");
                        s.push_str(&spec);
                        s.push('>');
                    }
                }
            } else {
                s.push(ch);
            }
        }
        Token::String(s)
    }

    pub(super) fn read_number(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        let mut has_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if ch == '.' && !has_dot {
                has_dot = true;
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Token::Number(s.parse().unwrap_or(0.0))
    }
}
