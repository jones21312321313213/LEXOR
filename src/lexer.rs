#![allow(dead_code)]

use regex::Regex;
use crate::token::{Token, TokenType};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    line: usize,

    // pre-compiled regexes
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

            re_whitespace: Regex::new(r"^[ \t\r]+").unwrap(),
            re_newline: Regex::new(r"^\n").unwrap(),
            re_comment: Regex::new(r"^%%[^\n]*").unwrap(),
            
            re_multi_keyword: Regex::new(r"^(SCRIPT AREA|START SCRIPT|END SCRIPT|START IF|END IF|ELSE IF|START FOR|END FOR|REPEAT WHEN|START REPEAT|END REPEAT)\b").unwrap(),
            
            re_number: Regex::new(r"^[0-9]+(\.[0-9]+)?").unwrap(),
            re_string: Regex::new(r#"^"[^"]*""#).unwrap(),
            re_char: Regex::new(r"^['’][^'’]['’]").unwrap(),
            
            re_keyword_or_id: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
            
          // Replace your existing re_symbol with this one:
            re_symbol: Regex::new(r"^(==|>=|<=|<>|!=|\+\+|--|\[.\]|\[\]|&&|\|\||!|\+|-|\*|/|%|&|<|>|=|\[|\]|\(|\)|,|:|\$)").unwrap(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while self.current < self.source.len() {
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::Eof, String::new(), self.line));
        
        self.tokens.clone()
    }

    fn scan_token(&mut self) {

        let remaining = &self.source[self.current..];

        // skip Whitespace
        if let Some(mat) = self.re_whitespace.find(remaining) {
            self.current += mat.end();
            return;
        }

        // new lines
        if let Some(mat) = self.re_newline.find(remaining) {
            self.current += mat.end();
            self.line += 1;
            self.tokens.push(Token::new(TokenType::NewLine, "\n".to_string(), self.line - 1));
            return;
        }

        // comments
        if let Some(mat) = self.re_comment.find(remaining) {
            self.current += mat.end();
            return;
        }

        // reserved keywords that are multi word like SCRIPT AREA, START FOR, START SCRIPT
        if let Some(mat) = self.re_multi_keyword.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            let token_type = self.check_keyword(&text).unwrap();
            self.tokens.push(Token::new(token_type, text, self.line));
            return;
        }

        // print: "this"
        if let Some(mat) = self.re_string.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            

            let value = text[1..text.len() - 1].to_string();
            

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

        // chars
        if let Some(mat) = self.re_char.find(remaining) {
            let text = mat.as_str().to_string();
            self.current += mat.end();
            

            let val = text.chars().nth(1).unwrap();
            self.tokens.push(Token::new(TokenType::CharLiteral(val), text, self.line));
            return;
        }

        //  numbers int and floats
        if let Some(mat) = self.re_number.find(remaining) {
            let text = mat.as_str().to_string();

            if let Some(next_char) = remaining[mat.end()..].chars().next() {
                if next_char.is_ascii_alphabetic() {
                    eprintln!("[Line {}] Error: Identifiers cannot start with a number.", self.line);
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

        // 8. single keywords or identifiers
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

        // symbols and operators
        if let Some(mat) = self.re_symbol.find(remaining) {
            let text = mat.as_str();
            self.current += mat.end();
            
            let token_type = match text {
                "++"=> TokenType::PlusPlus,
                "--"=> TokenType::MinusMinus,
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

        let bad_char = remaining.chars().next().unwrap();
        eprintln!("[Line {}] Error: Unexpected character '{}'", self.line, bad_char);
        self.current += bad_char.len_utf8(); 
    }

    // mapping of reserved key words
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