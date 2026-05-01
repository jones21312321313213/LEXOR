#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single char tokens
    LeftPar,
    RightPar,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Modulo,
    Star,
    Slash,
    Plus,
    Minus,
    Dollar,
    Ampersand,

    EscapeCode, // for []

    // comparison
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    NotEqual,

    //literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    BoolLiteral(bool),
    StringLiteral(String),

    // data types
    IntType,
    CharType,
    BoolType,
    FloatType,

    // logical operators
    And,
    Or,
    Not,

    // keywords
    ScriptArea,
    StartScript,
    EndScript,
    Declare,
    Print,
    Scan,

    // control flow
    If,
    StartIf,
    EndIf,
    Else,
    ElseIf,
    For,
    StartFor,
    EndFor,
    RepeatWhen,
    StartRepeat,
    EndRepeat,

    NewLine,
    Comment,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}
