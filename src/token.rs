/// The TokenType enum uses Rust's powerful sum types to store data
/// directly inside the variants. No more messy 'Object literal' casting!
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

    // Special LEXOR tokens
    EscapeCode, // for []

    // Comparison
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    EqualEqual,
    NotEqual,

    //Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    BoolLiteral(bool),
    StringLiteral(String),

    // Data types
    IntType,
    CharType,
    BoolType,
    FloatType,

    // Logical operators
    And,
    Or,
    Not,

    // Keywords
    ScriptArea,
    StartScript,
    EndScript,
    Declare,
    Print,
    Scan,

    // Control flow
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

/// The Token struct is now incredibly clean.
/// It only needs the type (which holds the data), the raw text, and the line number.
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    /// A helpful constructor to make creating tokens easier in your Scanner
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}
