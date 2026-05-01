use crate::token::Token;

#[derive(Debug, Clone)]

pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Line {}] Runtime Error at '{}': {}",
            self.token.line, self.token.lexeme, self.message
        )
    }
}

impl std::error::Error for RuntimeError {}
