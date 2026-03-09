use super::Lexer;
use super::token::Token;

impl<'a> Lexer<'a> {
    /// 実験的なトークンのパース試行
    pub(super) fn try_staging_tokens(&mut self, ch: char) -> Option<Token> {
        match ch {
            // 例: '@' を試験的に導入する場合
            // '@' => Some(Token::At),
            _ => None,
        }
    }
}
