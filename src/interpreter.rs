use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::expr::{Expr, LiteralValue};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};

pub struct Interpreter {
    // Wrap the global environment in Rc and RefCell so we can safely mutate it
    // and share it with block scopes.
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block { statements } => {
                // Create a new child environment that points to the current one
                let new_env = Rc::new(RefCell::new(Environment::new_with_enclosing(Rc::clone(
                    &self.environment,
                ))));
                self.execute_block(statements, new_env)
            }

            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_value = self.evaluate(condition)?;
                if self.is_truthy(&cond_value) {
                    self.execute(then_branch)
                } else if let Some(el_branch) = else_branch {
                    self.execute(el_branch)
                } else {
                    Ok(())
                }
            }

            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                print!("{}", self.stringify(&value));
                io::stdout().flush().unwrap(); // Ensure it prints immediately
                Ok(())
            }

            Stmt::Scan { variables, types } => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let parts: Vec<&str> = input.trim().split(',').collect();

                if parts.len() < variables.len() {
                    return Err(RuntimeError::new(
                        variables[0].clone(),
                        format!(
                            "insufficient input for SCAN. Expected {} values.",
                            variables.len()
                        ),
                    ));
                }

                for (i, var_token) in variables.iter().enumerate() {
                    let value_str = parts[i].trim();
                    let type_str = &types[i];
                    let value = self.cast_to_type(value_str, type_str, var_token)?;
                    self.environment.borrow_mut().assign(var_token, value)?;
                }
                Ok(())
            }

            Stmt::Var {
                data_type,
                name,
                initializer,
            } => {
                let mut value = match data_type.token_type {
                    TokenType::IntType => LiteralValue::Int(0),
                    TokenType::FloatType => LiteralValue::Float(0.0),
                    TokenType::BoolType => LiteralValue::Bool(false),
                    TokenType::CharType => LiteralValue::Char(' '),
                    _ => unreachable!(),
                };

                if let Some(init_expr) = initializer {
                    let init_val = self.evaluate(init_expr)?;

                    // Type Checking (Matches LEXOR strict typing)
                    match (&value, &init_val) {
                        (LiteralValue::Int(_), LiteralValue::Int(_)) => {}
                        (LiteralValue::Int(_), _) => {
                            return Err(RuntimeError::new(
                                name.clone(),
                                format!("type mismatch cannot initialize INT with {:?}", init_val),
                            ));
                        }
                        (LiteralValue::Float(_), LiteralValue::Float(_))
                        | (LiteralValue::Float(_), LiteralValue::Int(_)) => {}
                        (LiteralValue::Float(_), _) => {
                            return Err(RuntimeError::new(
                                name.clone(),
                                format!(
                                    "type mismatch cannot initialize FLOAT with {:?}",
                                    init_val
                                ),
                            ));
                        }
                        (LiteralValue::Bool(_), LiteralValue::Bool(_)) => {}
                        (LiteralValue::Bool(_), _) => {
                            return Err(RuntimeError::new(
                                name.clone(),
                                format!("type mismatch expected BOOL value."),
                            ));
                        }
                        (LiteralValue::Char(_), LiteralValue::Char(_)) => {}
                        (LiteralValue::Char(_), _) => {
                            return Err(RuntimeError::new(
                                name.clone(),
                                format!("type mismatch expected CHAR value."),
                            ));
                        }
                        _ => {}
                    }
                    value = init_val;
                }

                self.environment.borrow_mut().define(name, value)
            }

            Stmt::While { condition, body } => {
                loop {
                    // 1. Mutably borrow self to evaluate the condition
                    let cond_val = self.evaluate(condition)?;

                    // 2. The mutable borrow is over! Now immutably borrow self for is_truthy
                    if !self.is_truthy(&cond_val) {
                        break;
                    }

                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                let for_env = Rc::new(RefCell::new(Environment::new_with_enclosing(Rc::clone(
                    &self.environment,
                ))));
                let previous_env = Rc::clone(&self.environment);
                self.environment = for_env;

                let result = (|| -> Result<(), RuntimeError> {
                    self.evaluate(initializer)?;

                    loop {
                        let cond_val = self.evaluate(condition)?;
                        if !self.is_truthy(&cond_val) {
                            break;
                        }

                        self.execute(body)?;
                        self.evaluate(increment)?;
                    }

                    Ok(())
                })();

                self.environment = previous_env; // restore environment
                result
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let previous = Rc::clone(&self.environment);
        self.environment = env;

        // Execute all statements, but catch any errors so we can restore the environment before bubbling them up
        let result = (|| -> Result<(), RuntimeError> {
            for statement in statements {
                self.execute(statement)?;
            }
            Ok(())
        })();

        self.environment = previous; // Restore
        result
    }

    // --- EXPRESSION EVALUATION ---

    fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, RuntimeError> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),

            Expr::Grouping { expression } => self.evaluate(expression),

            Expr::Variable { name } => self.environment.borrow().get(name),

            Expr::Assign { name, value } => {
                let mut new_val = self.evaluate(value)?;
                let existing_val = self.environment.borrow().get(name)?;

                // Type Check and Cast
                match (&existing_val, &new_val) {
                    (LiteralValue::Int(_), LiteralValue::Float(f)) if f.fract() == 0.0 => {
                        new_val = LiteralValue::Int(*f as i64);
                    }
                    (LiteralValue::Int(_), LiteralValue::Int(_)) => {}
                    (LiteralValue::Int(_), _) => {
                        return Err(RuntimeError::new(
                            name.clone(),
                            format!(
                                "type mismatch cannot assign to INT variable '{}'.",
                                name.lexeme
                            ),
                        ));
                    }
                    (LiteralValue::Float(_), LiteralValue::Float(_))
                    | (LiteralValue::Float(_), LiteralValue::Int(_)) => {}
                    (LiteralValue::Float(_), _) => {
                        return Err(RuntimeError::new(
                            name.clone(),
                            format!(
                                "type mismatch cannot assign to FLOAT variable '{}'.",
                                name.lexeme
                            ),
                        ));
                    }
                    (LiteralValue::Bool(_), LiteralValue::Bool(_)) => {}
                    (LiteralValue::Bool(_), _) => {
                        return Err(RuntimeError::new(
                            name.clone(),
                            "type mismatch expected BOOL value.".to_string(),
                        ));
                    }
                    _ => {}
                }

                self.environment
                    .borrow_mut()
                    .assign(name, new_val.clone())?;
                Ok(new_val)
            }

            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;

                // Rust pattern matching handles the short-circuiting beautifully
                match operator.token_type {
                    TokenType::Or => {
                        if self.is_truthy(&left_val) {
                            return Ok(LiteralValue::Bool(true));
                        }
                    }
                    TokenType::And => {
                        if !self.is_truthy(&left_val) {
                            return Ok(LiteralValue::Bool(false));
                        }
                    }
                    _ => unreachable!(),
                }

                let right_val = self.evaluate(right)?;
                Ok(LiteralValue::Bool(self.is_truthy(&right_val)))
            }

            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Not => Ok(LiteralValue::Bool(!self.is_truthy(&right_val))),
                    TokenType::Minus => match right_val {
                        LiteralValue::Int(i) => Ok(LiteralValue::Int(-i)),
                        LiteralValue::Float(f) => Ok(LiteralValue::Float(-f)),
                        _ => Err(RuntimeError::new(
                            operator.clone(),
                            "operand must be a number.".to_string(),
                        )),
                    },
                    TokenType::Plus => match right_val {
                        LiteralValue::Int(i) => Ok(LiteralValue::Int(i)),
                        LiteralValue::Float(f) => Ok(LiteralValue::Float(f)),
                        _ => Err(RuntimeError::new(
                            operator.clone(),
                            "operand must be a number.".to_string(),
                        )),
                    },
                    _ => unreachable!(),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let l_val = self.evaluate(left)?;
                let r_val = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Ampersand => Ok(LiteralValue::String(format!(
                        "{}{}",
                        self.stringify(&l_val),
                        self.stringify(&r_val)
                    ))),
                    TokenType::EqualEqual => Ok(LiteralValue::Bool(self.is_equal(&l_val, &r_val))),
                    TokenType::NotEqual => Ok(LiteralValue::Bool(!self.is_equal(&l_val, &r_val))),
                    _ => {
                        // Math and Comparison operations
                        match (&l_val, &r_val) {
                            (LiteralValue::Int(l), LiteralValue::Int(r)) => {
                                match operator.token_type {
                                    TokenType::Plus => Ok(LiteralValue::Int(l + r)),
                                    TokenType::Minus => Ok(LiteralValue::Int(l - r)),
                                    TokenType::Star => Ok(LiteralValue::Int(l * r)),
                                    TokenType::Slash => {
                                        if *r == 0 {
                                            return Err(RuntimeError::new(
                                                operator.clone(),
                                                "Error, Division by 0".to_string(),
                                            ));
                                        }
                                        Ok(LiteralValue::Int(l / r))
                                    }
                                    TokenType::Modulo => {
                                        if *r == 0 {
                                            return Err(RuntimeError::new(
                                                operator.clone(),
                                                "Error, Modulo by 0".to_string(),
                                            ));
                                        }
                                        Ok(LiteralValue::Int(l % r))
                                    }
                                    TokenType::Greater => Ok(LiteralValue::Bool(l > r)),
                                    TokenType::GreaterEqual => Ok(LiteralValue::Bool(l >= r)),
                                    TokenType::Less => Ok(LiteralValue::Bool(l < r)),
                                    TokenType::LessEqual => Ok(LiteralValue::Bool(l <= r)),
                                    _ => unreachable!(),
                                }
                            }
                            (l, r) => {
                                // If either is a float, promote both to floats
                                let ld = self.as_double(l, operator)?;
                                let rd = self.as_double(r, operator)?;
                                match operator.token_type {
                                    TokenType::Plus => Ok(LiteralValue::Float(ld + rd)),
                                    TokenType::Minus => Ok(LiteralValue::Float(ld - rd)),
                                    TokenType::Star => Ok(LiteralValue::Float(ld * rd)),
                                    TokenType::Slash => {
                                        if rd == 0.0 {
                                            return Err(RuntimeError::new(
                                                operator.clone(),
                                                "Error, Division by 0".to_string(),
                                            ));
                                        }
                                        Ok(LiteralValue::Float(ld / rd))
                                    }
                                    TokenType::Modulo => {
                                        if rd == 0.0 {
                                            return Err(RuntimeError::new(
                                                operator.clone(),
                                                "Error, Modulo by 0".to_string(),
                                            ));
                                        }
                                        Ok(LiteralValue::Float(ld % rd))
                                    }
                                    TokenType::Greater => Ok(LiteralValue::Bool(ld > rd)),
                                    TokenType::GreaterEqual => Ok(LiteralValue::Bool(ld >= rd)),
                                    TokenType::Less => Ok(LiteralValue::Bool(ld < rd)),
                                    TokenType::LessEqual => Ok(LiteralValue::Bool(ld <= rd)),
                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- HELPER METHODS ---

    fn is_truthy(&self, val: &LiteralValue) -> bool {
        match val {
            LiteralValue::Nil => false,
            LiteralValue::Bool(b) => *b,
            _ => true,
        }
    }

    fn as_double(&self, val: &LiteralValue, operator: &Token) -> Result<f64, RuntimeError> {
        match val {
            LiteralValue::Int(i) => Ok(*i as f64),
            LiteralValue::Float(f) => Ok(*f),
            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operand must be a number.".to_string(),
            )),
        }
    }

    fn is_equal(&self, l: &LiteralValue, r: &LiteralValue) -> bool {
        match (l, r) {
            (LiteralValue::Int(li), LiteralValue::Int(ri)) => li == ri,
            (LiteralValue::Float(lf), LiteralValue::Float(rf)) => lf == rf,
            // Cross-type equality (like Java's Number cast)
            (LiteralValue::Int(li), LiteralValue::Float(rf)) => (*li as f64) == *rf,
            (LiteralValue::Float(lf), LiteralValue::Int(ri)) => *lf == (*ri as f64),
            (LiteralValue::String(ls), LiteralValue::String(rs)) => ls == rs,
            (LiteralValue::Bool(lb), LiteralValue::Bool(rb)) => lb == rb,
            (LiteralValue::Char(lc), LiteralValue::Char(rc)) => lc == rc,
            (LiteralValue::Nil, LiteralValue::Nil) => true,
            _ => false,
        }
    }

    fn stringify(&self, val: &LiteralValue) -> String {
        match val {
            LiteralValue::Nil => "null".to_string(),
            LiteralValue::Bool(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            LiteralValue::Int(i) => i.to_string(),
            LiteralValue::Float(f) => {
                if f.fract() == 0.0 {
                    (*f as i64).to_string()
                } else {
                    f.to_string()
                }
            }
            LiteralValue::String(s) => s.clone(),
            LiteralValue::Char(c) => c.to_string(),
        }
    }

    fn cast_to_type(
        &self,
        value_str: &str,
        type_str: &str,
        var_token: &Token,
    ) -> Result<LiteralValue, RuntimeError> {
        match type_str {
            "INT" => value_str
                .parse::<i64>()
                .map(LiteralValue::Int)
                .map_err(|_| {
                    RuntimeError::new(
                        var_token.clone(),
                        format!("type mismatch cannot convert '{}' to INT.", value_str),
                    )
                }),
            "FLOAT" => value_str
                .parse::<f64>()
                .map(LiteralValue::Float)
                .map_err(|_| {
                    RuntimeError::new(
                        var_token.clone(),
                        format!("type mismatch cannot convert '{}' to FLOAT.", value_str),
                    )
                }),
            "BOOL" => {
                if value_str.eq_ignore_ascii_case("true") {
                    Ok(LiteralValue::Bool(true))
                } else if value_str.eq_ignore_ascii_case("false") {
                    Ok(LiteralValue::Bool(false))
                } else {
                    Err(RuntimeError::new(
                        var_token.clone(),
                        format!(
                            "expected BOOL (TRUE/FALSE) for '{}', got: {}",
                            var_token.lexeme, value_str
                        ),
                    ))
                }
            }
            "CHAR" => {
                if value_str.chars().count() == 1 {
                    Ok(LiteralValue::Char(value_str.chars().next().unwrap()))
                } else {
                    Err(RuntimeError::new(
                        var_token.clone(),
                        format!("expected single CHAR for '{}'.", var_token.lexeme),
                    ))
                }
            }
            _ => Err(RuntimeError::new(
                var_token.clone(),
                "unknown type for SCAN.".to_string(),
            )),
        }
    }
}
