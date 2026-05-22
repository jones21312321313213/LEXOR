use std::collections::HashMap;

use crate::expr::{Expr, LiteralValue};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};

/**
 *
 *
 * assignment -> concatenation (&) -> or -> and -> logicalNot (NOT)
 * -> equality (==, <>) -> comparison (>, <, etc) -> term (+, -)
 * -> factor (*, /, %) -> unary (+, -, !) -> primary
 */

/** cfg for EXPR(expressions)
 * <expression>     ::= <assignment>
 * <assignment>     ::= <identifier> "=" <assignment> | <concatenation>
 * <concatenation>  ::= <or> ( "&" <or> )*
 * <or>             ::= <and> ( "OR" <and> )*
 * <and>            ::= <not> ( "AND" <not> )*
 * <not>            ::= "NOT" <not> | <equality>
 * <equality>       ::= <comparison> ( ( "==" | "<>" ) <comparison> )*
 * <comparison>     ::= <term> ( ( ">" | ">=" | "<" | "<=" ) <term> )*
 * <term>           ::= <factor> ( ( "+" | "-" ) <factor> )*
 * <factor>         ::= <unary> ( ( "*" | "/" | "%" ) <unary> )*
 * <unary>          ::= ( "+" | "-" ) <unary> | <primary>
 * <primary>        ::= NUMBER | STRING | BOOLEAN | IDENTIFIER | "(" <expression> ")"
 */

// Sentinel struct to unwind the parser
#[derive(Debug)]
pub struct ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    variable_types: HashMap<String, TokenType>,
    allow_declarations: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            variable_types: HashMap::new(),
            allow_declarations: true,
        }
    }

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        self.current = 0;
        self.allow_declarations = true;
        self.variable_types.clear();

        let mut statements = Vec::new();
        let mut had_error = false; // NEW: Track if we hit any syntax errors

        self.swallow_new_lines();
        if self.is_at_end() {
            return Some(statements);
        }

        // Boundary checks
        if self
            .consume(
                TokenType::ScriptArea,
                "Expect 'SCRIPT AREA' at the beginning of the script.",
            )
            .is_err()
        {
            had_error = true;
            self.synchronize();
        }
        self.swallow_new_lines();
        if self
            .consume(
                TokenType::StartScript,
                "Expect 'START SCRIPT' after 'SCRIPT AREA'.",
            )
            .is_err()
        {
            had_error = true;
            self.synchronize();
        }
        if self
            .consume(TokenType::NewLine, "Expect newline after 'START SCRIPT'.")
            .is_err()
        {
            had_error = true;
            self.synchronize();
        }

        // 3. Parse declarations and statements until we hit END SCRIPT
        while !self.check(&TokenType::EndScript) && !self.is_at_end() {
            self.swallow_new_lines();

            match self.declaration() {
                Ok(decl_list) => statements.extend(decl_list),
                Err(_) => {
                    had_error = true; // NEW: We caught an error!
                    self.synchronize();
                }
            }
        }

        // 4. Close the script boundary
        if self
            .consume(
                TokenType::EndScript,
                "Expect 'END SCRIPT' at the end of the script.",
            )
            .is_err()
        {
            had_error = true;
            self.synchronize();
        }

        // Check for trailing garbage
        self.swallow_new_lines();
        if !self.is_at_end() {
            had_error = true; // NEW: Flag trailing garbage as an error
            let peek_token = self.peek().clone();
            self.error(
                &peek_token,
                "Unexpected code found after 'END SCRIPT'. Only one script block is allowed.",
            );
        }

        // CRITICAL FIX: If there were any errors, abort and return None!
        if had_error { None } else { Some(statements) }
    }

    // --- EXPRESSIONS ---

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.concatenation()?;

        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(self.error(&equals, "Invalid assignment target."));
        }
        Ok(expr)
    }

    fn concatenation(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.or()?;
        while self.match_tokens(&[TokenType::Ampersand]) {
            let operator = self.previous().clone();
            let right = self.or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?; // Call the next level up
        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_not()?;
        while self.match_tokens(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.logical_not()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logical_not(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenType::Not]) {
            let operator = self.previous().clone();
            let right = self.logical_not()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_tokens(&[TokenType::NotEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Modulo]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.unary()?; // recursive call to parse the operand
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        // self.primary()
        self.postfix()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        match &token.token_type {
            TokenType::IntLiteral(val) => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Int(*val),
                })
            }
            TokenType::FloatLiteral(val) => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Float(*val),
                })
            }
            TokenType::StringLiteral(val) => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::String(val.clone()),
                })
            }
            TokenType::CharLiteral(val) => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Char(*val),
                })
            }
            TokenType::BoolLiteral(val) => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Bool(*val),
                })
            }
            TokenType::Dollar => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::String("\n".to_string()),
                })
            }
            TokenType::EscapeCode => {
                self.advance();

                // Strip the surrounding '[' and ']'
                // "[[]" becomes "[", "[]]" becomes "]", and "[]" becomes ""
                let inner_text = if token.lexeme.len() > 2 {
                    token.lexeme[1..token.lexeme.len() - 1].to_string()
                } else {
                    String::new()
                };

                Ok(Expr::Literal {
                    value: LiteralValue::String(inner_text),
                })
            }
            TokenType::Identifier(_) => {
                self.advance();
                Ok(Expr::Variable { name: token })
            }
            TokenType::LeftPar => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightPar, "Expect ')' after expression.")?;
                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            }
            _ => Err(self.error(&token, "Expect expression.")),
        }
    }

    // --- STATEMENTS ---
    fn statement(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut list = Vec::new();
        self.swallow_new_lines();

        if self.check(&TokenType::EndIf)
            || self.check(&TokenType::Else)
            || self.check(&TokenType::ElseIf)
            || self.check(&TokenType::EndFor)
            || self.check(&TokenType::EndRepeat)
            || self.check(&TokenType::EndScript)
        {
            let peek_token = self.peek().clone();
            return Err(self.error(
                &peek_token,
                &format!(
                    "Unexpected '{}' found. Missing matching START block.",
                    peek_token.lexeme
                ),
            ));
        }

        if self.match_tokens(&[TokenType::NewLine]) {
            return Ok(list);
        }

        if self.match_tokens(&[TokenType::Print]) {
            list.push(self.print_statement()?);
        } else if self.match_tokens(&[TokenType::Scan]) {
            list.push(self.scan_statement()?);
        } else if self.match_tokens(&[TokenType::If]) {
            list.push(self.if_statement()?);
        } else if self.match_tokens(&[TokenType::RepeatWhen]) {
            list.push(self.repeat_when_statement()?);
        } else if self.match_tokens(&[TokenType::For]) {
            list.push(self.for_statement()?);
        } else {
            list.push(self.expression_statement()?);
        }

        self.swallow_new_lines();
        Ok(list)
    }

    fn declaration(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.swallow_new_lines();

        if self.check(&TokenType::EndScript) || self.is_at_end() {
            return Ok(Vec::new());
        }

        if self.match_tokens(&[TokenType::Declare]) {
            if !self.allow_declarations {
                let token = self.previous().clone();
                return Err(self.error(&token, "Declarations must come before executable code."));
            }
            return self.var_declaration();
        }

        self.allow_declarations = false;
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let type_token = self.consume_any_type("Expect data type after 'DECLARE'.")?;
        let mut declarations = Vec::new();

        if self.check(&TokenType::NewLine) {
            let peek_token = self.peek().clone();
            return Err(self.error(
                &peek_token,
                &format!("Missing variable name after '{}'.", type_token.lexeme),
            ));
        }

        // custom check to make sure identifier sya
        if !matches!(self.peek().token_type, TokenType::Identifier(_)) {
            let peek_token = self.peek().clone();
            return Err(self.error(
                &peek_token,
                &format!(
                    "Reserved keyword '{}' cannot be used as a variable name.",
                    peek_token.lexeme
                ),
            ));
        }

        loop {
            let name = self.consume_identifier("Expect variable name.")?;

            if self.variable_types.contains_key(&name.lexeme) {
                return Err(self.error(
                    &name,
                    &format!("Variable '{}' is already defined.", name.lexeme),
                ));
            }

            // SAVE THE TYPE TO THE MAP
            self.variable_types
                .insert(name.lexeme.clone(), type_token.token_type.clone());

            let mut initializer = None;
            if self.match_tokens(&[TokenType::Equal]) {
                if self.check(&TokenType::NewLine) {
                    let peek_token = self.peek().clone();
                    return Err(self.error(
                        &peek_token,
                        &format!(
                            "Variable '{}' requires an initial value after '='.",
                            name.lexeme
                        ),
                    ));
                }
                initializer = Some(self.expression()?);
            }

            declarations.push(Stmt::Var {
                data_type: type_token.clone(),
                name,
                initializer,
            });

            if !self.match_tokens(&[TokenType::Comma]) {
                break;
            }
        }

        self.swallow_new_lines();
        Ok(declarations)
    }

    fn scan_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Colon, "Expect ':' after 'SCAN'.")?;
        let mut variables = Vec::new();
        let mut types = Vec::new();

        loop {
            let var_token = self.consume_identifier("Expect variable name for SCAN.")?;
            variables.push(var_token.clone());

            if let Some(t_type) = self.variable_types.get(&var_token.lexeme) {
                let type_name = match t_type {
                    TokenType::IntType => "INT".to_string(),
                    TokenType::FloatType => "FLOAT".to_string(),
                    TokenType::BoolType => "BOOL".to_string(),
                    TokenType::CharType => "CHAR".to_string(),
                    _ => unreachable!(),
                };
                types.push(type_name);
            } else {
                return Err(self.error(
                    &var_token,
                    &format!(
                        "Variable '{}' must be declared before SCAN.",
                        var_token.lexeme
                    ),
                ));
            }

            if !self.match_tokens(&[TokenType::Comma]) {
                break;
            }
        }

        if !self.is_at_end() && self.check(&TokenType::NewLine) {
            self.advance();
        }

        Ok(Stmt::Scan { variables, types })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        // Safety check
        if !self.is_at_end()
            && !self.check(&TokenType::NewLine)
            && !self.check(&TokenType::EndScript)
            && !self.check(&TokenType::EndIf)
            && !self.check(&TokenType::EndFor)
            && !self.check(&TokenType::EndRepeat)
            && !self.check(&TokenType::Else)
            && !self.check(&TokenType::ElseIf)
        {
            let peek_token = self.peek().clone();
            return Err(self.error(&peek_token, "Expect new line after statement."));
        }

        self.swallow_new_lines();
        Ok(Stmt::Expression { expression: expr })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::Colon, "Expect ':' after 'PRINT'.")?;
        let value = self.expression()?;
        self.swallow_new_lines();
        Ok(Stmt::Print { expression: value })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftPar, "Expect '(' after 'IF'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightPar, "Expect ')' after IF condition.")?;
        self.swallow_new_lines();

        self.consume(TokenType::StartIf, "Expect 'START IF' to begin block.")?;
        self.swallow_new_lines();
        let then_branch = Box::new(self.block(TokenType::EndIf)?);
        self.swallow_new_lines();

        let mut else_branch = None;

        // handle else if
        if self.match_tokens(&[TokenType::ElseIf]) {
            else_branch = Some(Box::new(self.if_statement()?));
        }
        // handle else
        else if self.match_tokens(&[TokenType::Else]) {
            self.swallow_new_lines();
            self.consume(TokenType::StartIf, "Expect 'START IF' after 'ELSE'.")?;
            self.swallow_new_lines();
            else_branch = Some(Box::new(self.block(TokenType::EndIf)?));
            self.swallow_new_lines();
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn repeat_when_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftPar, "Expect '(' after 'REPEAT WHEN'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightPar, "Expect ')' after condition.")?;
        self.swallow_new_lines();

        self.consume(TokenType::StartRepeat, "Expect 'START REPEAT'.")?;
        self.swallow_new_lines();
        let body = Box::new(self.block(TokenType::EndRepeat)?);
        self.swallow_new_lines();

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftPar, "Expect '(' after 'FOR'.")?;

        let initializer = self.expression()?;
        self.consume(TokenType::Comma, "Expect ',' after initializer.")?;

        let condition = self.expression()?;
        self.consume(TokenType::Comma, "Expect ',' after condition.")?;

        let increment = self.expression()?;
        self.consume(TokenType::RightPar, "Expect ')' after FOR clauses.")?;
        self.swallow_new_lines();

        self.consume(TokenType::StartFor, "Expect 'START FOR'.")?;
        self.swallow_new_lines();

        let body = Box::new(self.block(TokenType::EndFor)?);
        self.swallow_new_lines();

        Ok(Stmt::For {
            initializer,
            condition,
            increment,
            body,
        })
    }

    fn block(&mut self, end_token: TokenType) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&end_token) && !self.is_at_end() {
            if self.check(&TokenType::EndScript) {
                let peek_token = self.peek().clone();
                return Err(self.error(
                    &peek_token,
                    &format!("Missing '{:?}' before END SCRIPT.", end_token),
                ));
            }
            let stmts = self.statement()?;
            statements.extend(stmts);
            self.swallow_new_lines();
        }

        self.consume(
            end_token.clone(),
            &format!("Expect {:?} to close block.", end_token),
        )?;
        Ok(Stmt::Block { statements })
    }

    // --- helper methods ---

    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let expr = self.primary()?;

        if self.match_tokens(&[TokenType::PlusPlus, TokenType::MinusMinus]) {
            let operator = self.previous().clone();

            if let Expr::Variable { name } = &expr {
                // Desugar "i++" into "i = i + 1"
                let synthetic_op = if operator.token_type == TokenType::PlusPlus {
                    Token::new(TokenType::Plus, "+".to_string(), operator.line)
                } else {
                    Token::new(TokenType::Minus, "-".to_string(), operator.line)
                };

                let one = Expr::Literal {
                    value: LiteralValue::Int(1),
                };

                let binary = Expr::Binary {
                    left: Box::new(expr.clone()),
                    operator: synthetic_op,
                    right: Box::new(one),
                };

                // Return an Assignment AST Node
                return Ok(Expr::Assign {
                    name: name.clone(),
                    value: Box::new(binary),
                });
            } else {
                return Err(self.error(
                    &operator,
                    "Invalid target for increment/decrement operator.",
                ));
            }
        }

        Ok(expr)
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        if token.token_type == TokenType::Eof {
            eprintln!("[Line {}] Error at end: {}", token.line, message);
        } else {
            eprintln!(
                "[Line {}] Error at '{}': {}",
                token.line, token.lexeme, message
            );
        }
        ParseError
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Helper dedicated entirely to safely grabbing an identifier payload.
    fn consume_identifier(&mut self, message: &str) -> Result<Token, ParseError> {
        if matches!(self.peek().token_type, TokenType::Identifier(_)) {
            Ok(self.advance().clone())
        } else {
            let token = self.peek().clone();

            // NEW: Smarter error messaging based on the token type
            let error_msg = match token.token_type {
                // If it's a known reserved word, give the strict keyword error
                TokenType::IntType
                | TokenType::FloatType
                | TokenType::BoolType
                | TokenType::CharType
                | TokenType::Declare
                | TokenType::Print
                | TokenType::Scan
                | TokenType::If
                | TokenType::Else
                | TokenType::ElseIf
                | TokenType::For
                | TokenType::And
                | TokenType::Or
                | TokenType::Not
                | TokenType::ScriptArea
                | TokenType::StartScript
                | TokenType::EndScript
                | TokenType::RepeatWhen
                | TokenType::StartRepeat
                | TokenType::EndRepeat
                | TokenType::StartIf
                | TokenType::EndIf
                | TokenType::StartFor
                | TokenType::EndFor => {
                    format!(
                        "Reserved keyword '{}' cannot be used as a variable name.",
                        token.lexeme
                    )
                }
                // If it's a symbol like '=', just give the standard missing variable message
                _ => format!("{} Got '{}' instead.", message, token.lexeme),
            };

            Err(self.error(&token, &error_msg))
        }
    }

    // checking data types
    fn consume_any_type(&mut self, message: &str) -> Result<Token, ParseError> {
        match self.peek().token_type {
            TokenType::IntType
            | TokenType::FloatType
            | TokenType::BoolType
            | TokenType::CharType => Ok(self.advance().clone()),
            _ => {
                let token = self.peek().clone();
                Err(self.error(&token, message))
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }
        let token = self.peek().clone();
        Err(self.error(&token, message))
    }

    fn swallow_new_lines(&mut self) {
        while self.match_tokens(&[TokenType::NewLine, TokenType::Comment]) {}
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::NewLine {
                return;
            }

            match self.peek().token_type {
                TokenType::Declare
                | TokenType::Print
                | TokenType::Scan
                | TokenType::If
                | TokenType::For
                | TokenType::RepeatWhen
                | TokenType::EndScript => return,
                _ => {}
            }
            self.advance();
        }
    }
}
