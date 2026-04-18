#![allow(dead_code)]

use regex::Regex;
use crate::token::{Token, TokenType};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    line: usize,

    // Pre-compiled regexes (Compiled once when the Lexer is created for speed)
    re_whitespace: Regex,
    re_newline: Regex,
    re_comment: Regex,
    re_multi_keyword: Regex,
    re_keyword_or_id: Regex,
    re_number: Regex,
    re_string: Regex,
    re_char: Regex,
    re_symbol: Regex,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: Vec::new(),
            current: 0,
            line: 1,

            // The '^' symbol ensures we only match at the exact FRONT of the remaining string
            re_whitespace: Regex::new(r"^[ \t\r]+").unwrap(),
            re_newline: Regex::new(r"^\n").unwrap(),
            re_comment: Regex::new(r"^%%[^\n]*").unwrap(),
            
            // \b ensures word boundaries (so "START SCRIPTING" doesn't match "START SCRIPT")
            re_multi_keyword: Regex::new(r"^(SCRIPT AREA|START SCRIPT|END SCRIPT|START IF|END IF|ELSE IF|START FOR|END FOR|REPEAT WHEN|START REPEAT|END REPEAT)\b").unwrap(),
            
            re_number: Regex::new(r"^[0-9]+(\.[0-9]+)?").unwrap(),
            re_string: Regex::new(r#"^"[^"]*""#).unwrap(),
            re_char: Regex::new(r"^['’][^'’]['’]").unwrap(),
            
            re_keyword_or_id: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
            
            // Matches all single and double character operators. 
            // Sorted longest to shortest so '==' is checked before '='
           re_symbol: Regex::new(r"^(==|>=|<=|<>|!=|\[.\]|\[\]|&&|\|\||!|\+|-|\*|/|%|&|<|>|=|\[|\]|\(|\)|,|:|\$)").unwrap(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while self.current < self.source.len() {
            self.scan_token();
        }

        // Add the EOF token at the end
        self.tokens.push(Token::new(TokenType::Eof, String::new(), self.line));
        
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        // Slice the string from our current position to the end
        let remaining = &self.source[self.current..];

        // 1. Skip Whitespace
        if let Some(mat) = self.re_whitespace.find(remaining) {
            self.current += mat.end();
            return;
        }

        // 2. Handle Newlines (We track this so error lines are accurate)
        if let Some(mat) = self.re_newline.find(remaining) {
            self.current += mat.end();
            self.line += 1;
            self.tokens.push(Token::new(TokenType::NewLine, "\n".to_string(), self.line - 1));
            return;
        }

        // 3. Skip Comments (%%)
        if let Some(mat) = self.re_comment.find(remaining) {
            self.current += mat.end();
            return;
        }

        // 4. Multi-Word Keywords ("SCRIPT AREA", "START FOR", etc.)
        if let Some(mat) = self.re_multi_keyword.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            let token_type = self.check_keyword(&text).unwrap();
            self.tokens.push(Token::new(token_type, text, self.line));
            return;
        }

        // 5. Strings
        if let Some(mat) = self.re_string.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            
            // Remove the quotes around the string
            let value = text[1..text.len() - 1].to_string();
            
            // If the string spanned multiple lines, update our line tracker
            self.line += value.matches('\n').count();

            if value == "TRUE" {
                self.tokens.push(Token::new(TokenType::BoolLiteral(true), text, self.line));
            } else if value == "FALSE" {
                self.tokens.push(Token::new(TokenType::BoolLiteral(false), text, self.line));
            } else {
                self.tokens.push(Token::new(TokenType::StringLiteral(value), text, self.line));
            }
            return;
        }

        // 6. Characters
        if let Some(mat) = self.re_char.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            
            // Extract the actual char inside the quotes
            let val = text.chars().nth(1).unwrap();
            self.tokens.push(Token::new(TokenType::CharLiteral(val), text, self.line));
            return;
        }

        // 7. Numbers (Ints and Floats)
        if let Some(mat) = self.re_number.find(remaining) {
            let text = mat.as_str().to_string();

            // CRITICAL CHECK: Prevent identifiers starting with numbers (e.g., "1stValue")
            if let Some(next_char) = remaining[mat.end()..].chars().next() {
                if next_char.is_ascii_alphabetic() {
                    eprintln!("[Line {}] Error: Identifiers cannot start with a number.", self.line);
                    // Fast-forward past the garbage word to recover gracefully
                    if let Some(id_mat) = self.re_keyword_or_id.find(&remaining[mat.end()..]) {
                        self.current += mat.end() + id_mat.end();
                    } else {
                        self.current += mat.end();
                    }
                    return;
                }
            }

            self.current += mat.end();
            
            if text.contains('.') {
                let val: f64 = text.parse().unwrap();
                self.tokens.push(Token::new(TokenType::FloatLiteral(val), text, self.line));
            } else {
                let val: i64 = text.parse().unwrap();
                self.tokens.push(Token::new(TokenType::IntLiteral(val), text, self.line));
            }
            return;
        }

        // 8. Single Keywords or Identifiers
        if let Some(mat) = self.re_keyword_or_id.find(remaining) {
            let text: String = mat.as_str().to_string();
            self.current += mat.end();
            
            if let Some(token_type) = self.check_keyword(&text) {
                self.tokens.push(Token::new(token_type, text, self.line));
            } else if text == "TRUE" {
                self.tokens.push(Token::new(TokenType::BoolLiteral(true), text, self.line));
            } else if text == "FALSE" {
                self.tokens.push(Token::new(TokenType::BoolLiteral(false), text, self.line));
            } else {
                self.tokens.push(Token::new(TokenType::Identifier(text.clone()), text, self.line));
            }
            return;
        }

        // 9. Symbols and Operators
        if let Some(mat) = self.re_symbol.find(remaining) {
            let text = mat.as_str();
            self.current += mat.end();
            
            let token_type = match text {
                "(" => TokenType::LeftPar,
                ")" => TokenType::RightPar,
                "[" => TokenType::LeftBracket,
                "]" => TokenType::RightBracket,
                "," => TokenType::Comma,
                ":" => TokenType::Colon,
                "%" => TokenType::Modulo,
                "*" => TokenType::Star,
                "/" => TokenType::Slash,
                "+" => TokenType::Plus,
                "-" => TokenType::Minus,
                "$" => TokenType::Dollar,
                "&" => TokenType::Ampersand,
                ">" => TokenType::Greater,
                ">=" => TokenType::GreaterEqual,
                "<" => TokenType::Less,
                "<=" => TokenType::LessEqual,
                "=" => TokenType::Equal,
                "==" => TokenType::EqualEqual,
                "!=" | "<>" => TokenType::NotEqual,
                "!" => TokenType::Not,
                _ if text.starts_with('[') && text.ends_with(']') => TokenType::EscapeCode,
                _ => unreachable!(),
            };
            
            self.tokens.push(Token::new(token_type, text.to_string(), self.line));
            return;
        }

        // FALLBACK: If absolutely nothing matches, we hit an unknown character
        let bad_char = remaining.chars().next().unwrap();
        eprintln!("[Line {}] Error: Unexpected character '{}'", self.line, bad_char);
        self.current += bad_char.len_utf8(); // Advance past it so we don't infinite loop
    }

    /// Exact mapping of your reserved keywords
    fn check_keyword(&self, text: &str) -> Option<TokenType> {
        match text {
            "DECLARE" => Some(TokenType::Declare),
            "SCRIPT AREA" => Some(TokenType::ScriptArea),
            "START SCRIPT" => Some(TokenType::StartScript),
            "END SCRIPT" => Some(TokenType::EndScript),
            "PRINT" => Some(TokenType::Print),
            "SCAN" => Some(TokenType::Scan),
            "INT" => Some(TokenType::IntType),
            "BOOL" => Some(TokenType::BoolType),
            "CHAR" => Some(TokenType::CharType),
            "FLOAT" => Some(TokenType::FloatType),
            "AND" => Some(TokenType::And),
            "OR" => Some(TokenType::Or),
            "NOT" => Some(TokenType::Not),
            "IF" => Some(TokenType::If),
            "START IF" => Some(TokenType::StartIf),
            "END IF" => Some(TokenType::EndIf),
            "ELSE" => Some(TokenType::Else),
            "ELSE IF" => Some(TokenType::ElseIf),
            "FOR" => Some(TokenType::For),
            "START FOR" => Some(TokenType::StartFor),
            "END FOR" => Some(TokenType::EndFor),
            "REPEAT WHEN" => Some(TokenType::RepeatWhen),
            "START REPEAT" => Some(TokenType::StartRepeat),
            "END REPEAT" => Some(TokenType::EndRepeat),
            _ => None,
        }
    }
}