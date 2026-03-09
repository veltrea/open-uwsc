pub mod token;
pub mod keywords;
pub mod literals;
pub mod operators;
#[cfg(feature = "experimental")]
pub mod staging;

pub use token::Token;

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    column: usize,
    buffer: std::collections::VecDeque<(Token, usize)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
            buffer: std::collections::VecDeque::new(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    pub fn next_token(&mut self) -> (Token, usize) {
        loop {
            if let Some(token) = self.buffer.pop_front() {
                return token;
            }
            self.skip_whitespace();

            let line = self.line;
            let ch = match self.advance() {
                Some(ch) => ch,
                None => return (Token::EOF, self.line),
            };

            // Simplified dispatch to modules
            let tok = match ch {
                '$' => {
                    // Hexadecimal number
                    let mut hex = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() {
                            hex.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    Token::Number(i64::from_str_radix(&hex, 16).unwrap_or(0) as f64)
                }
                '<' | '>' | '+' | '-' | '*' | '/' | '=' | '!' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '.' => {
                    self.read_operator(ch)
                }
                c if c.is_ascii_digit() => self.read_number(c),
                c if c.is_alphabetic() || c == '_' => self.read_identifier(c),
                '"' => self.read_string(),
                '0'..='9' => self.read_number(ch),
                _ if ch.is_alphabetic() => self.read_identifier(ch),
                '\n' => Token::Newline,
                '\r' => {
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                    Token::Newline
                }
                _ => {
                    #[cfg(feature = "experimental")]
                    if let Some(tok) = self.try_staging_tokens(ch) {
                        tok
                    } else {
                        self.read_operator(ch)
                    }
                    #[cfg(not(feature = "experimental"))]
                    self.read_operator(ch)
                }
            };

            // If it's a "virtual" token that means skip, continue loop
            // For now, let's use LParen as a dummy to check if it's working
            // Actually, I'll need a real Skip token or change read_operator to return Option<Token>
            match tok {
                Token::Ref => continue, // We'll use Ref as Skip for now
                _ => return (tok, line),
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }
}
