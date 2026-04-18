use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::RuntimeError;
use crate::expr::LiteralValue;
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Environment {
    // Rc<RefCell<...>> is the standard Rust pattern for interpreters to allow
    // multiple child scopes to safely point to and mutate a parent scope.
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    //creates the global environment
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name_token: &Token, value: LiteralValue) -> Result<(), RuntimeError> {
        let name = &name_token.lexeme;

        if self.values.contains_key(name) {
            // This prevents the "20.5" from overwriting the "10"
            return Err(RuntimeError::new(
                name_token.clone(),
                format!("Variable '{}' is already defined in this scope.", name),
            ));
        }

        self.values.insert(name.clone(), value);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<LiteralValue, RuntimeError> {
        // check the current scope
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        //if not here then the parent scope
        if let Some(enclosing) = &self.enclosing {
            // .borrow() safely opens the RefCell to read the parent data
            return enclosing.borrow().get(name);
        }
        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: LiteralValue) -> Result<(), RuntimeError> {
        // check if it exists in the current scope
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        // if not here try to assign in the parent scope
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
