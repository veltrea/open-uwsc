#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    If,
    IfB,
    Then,
    Else,
    ElseIf,
    EndIf,
    For,
    To,
    Step,
    In,
    Next,
    While,
    Wend,
    Repeat,
    Until,
    Select,
    Case,
    Default,
    SelEnd,
    Procedure,
    Function,
    Fend,
    Module,
    EndModule,
    Class,
    EndClass,
    Enum,
    EndEnum,
    Dim,
    Public,
    Const,
    Hashtbl,
    Try,
    Finally,
    Except,
    EndTry,
    With,
    EndWith,
    TextBlock,
    EndTextBlock,
    Call,
    Break,
    Continue,
    Exit,
    ExitExit,
    Print,
    Option,
    DefDll,
    Thread,
    Result,
    Var,
    Global,
    This,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),
    Boolean(bool),
    SpecialConst(String),
    Null,
    Empty,
    Nothing,

    // Special Characters & Operators
    Assign,      // =
    AddAssign,   // +=
    SubAssign,   // -=
    MulAssign,   // *=
    DivAssign,   // /=
    ModAssign,   // MOD=
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Mod,         // MOD
    Equal,       // == or = in predicates
    NotEqual,    // <> or !=
    Greater,     // >
    Less,        // <
    GreaterEq,   // >=
    LessEq,      // <=
    And,         // AND
    Or,          // OR
    Xor,         // XOR
    Not,         // !
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    LBrace,      // {
    RBrace,      // }
    Comma,       // ,
    Colon,       // :
    Semicolon,   // ;
    Dot,         // .
    Underscore,  // _ (line continuation)
    Ref,         // & (reference, though UWSC uses it differently)
    
    // Line endings
    Newline,
    EOF,
}
