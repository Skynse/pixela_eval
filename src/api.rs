use std::collections::HashMap;

pub use super::expression::Expression;
pub use super::parser::{Parser, Token};

pub fn eval(input: String, x: Option<f64>) -> Option<f64> {
    let mut expr = Expression::new(input);
    let mut variables = HashMap::new();
    variables.insert("x".to_string(), x.unwrap_or(0.0));
    expr.eval_with_var()
}
