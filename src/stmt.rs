use crate::expr::Expr;
use crate::token::Token;

/// This single enum replaces Stmt, Block, Expression, For, If, Print, Scan, Var, and While!
#[derive(Debug, Clone)]
pub enum Stmt {
    //Block: For START/END blocks like START IF, START FOR, etc.
    Block {
        statements: Vec<Stmt>,
    },

    //Expression Statement: For standalone assignments like x = y = 4
    Expression {
        expression: Expr,
    },

    // FOR Statement: Specialized for the FOR(initialization, condition, update)
    For {
        initializer: Expr,
        condition: Expr,
        increment: Expr,
        body: Box<Stmt>,
    },

    // IF Statement: For IF, ELSE IF, and ELSE control flows
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        // In Java, this might be null if there is no ELSE. In Rust, we use Option!
        else_branch: Option<Box<Stmt>>,
    },

    // PRINT Statement: Formatted output for data and escape codes
    Print {
        expression: Expr,
    },

    // SCAN Statement: Allowing user to input a value to a data type
    Scan {
        variables: Vec<Token>,
        types: Vec<String>,
    },

    //  VAR Statement: Handles DECLARE for INT, CHAR, BOOL, and FLOAT
    Var {
        // 'type' is a reserved keyword in Rust, so we name it 'data_type'
        data_type: Token,
        name: Token,
        // Using Option because a user might DECLARE a variable without assigning a value right away
        initializer: Option<Expr>,
    },

    // While (REPEAT WHEN) Statement
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
