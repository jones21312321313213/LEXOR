// Adjust this import path depending on where your Token struct lives!
use crate::token::Token;

/// Replaces Java's `Object` for Literal values.
/// Since Rust is strictly typed, we define exactly what a literal can be.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Nil,
}

/// The AST Expression node.
/// This single enum replaces Expr.java, Assign.java, Binary.java, Grouping.java,
/// Literal.java, Logical.java, Unary.java, and Variable.java!
#[derive(Debug, Clone)]
pub enum Expr {
    // Handles variable assignment (e.g., x = 5)
    Assign {
        name: Token,
        value: Box<Expr>,
    },

    // Handles arithmetic and comparison operators (*, /, %, +, -, >, <, ==, <>, >=, <=)
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    // Handles code inside parenthesis ()
    Grouping {
        expression: Box<Expr>,
    },

    // Handles INT, FLOAT, CHAR, and BOOL values
    Literal {
        value: LiteralValue,
    },

    // Handles boolean logic (AND, OR)
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    // Handles unary positive and negative (+, -) and logical NOT
    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    // Handles variable usage after DECLARE
    Variable {
        name: Token,
    },
}
