use std::collections::HashMap;

use super::parser::{Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    data: Vec<Token>,
}

impl Stack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, t: Token) {
        self.data.push(t);
    }

    fn _peek(&self) -> Option<&Token> {
        //  get the last element of the stack
        self.data.last()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    input: String,
    pub stack: Stack,
}

pub trait Sanitize {
    fn sanitize(&self) -> String;
}

impl Sanitize for String {
    fn sanitize(&self) -> String {
        let mut sanitized = String::new();
        let mut prev_char = ' ';

        for (i, c) in self.chars().enumerate() {
            if c.is_ascii_alphanumeric() {
                if i > 0 && (prev_char.is_ascii_digit() || prev_char == ')') {
                    sanitized.push('*');
                }
            }
            sanitized.push(c);
            prev_char = c;
        }

        sanitized
    }
}

impl Expression {
    pub fn new(input: String) -> Self {
        Self {
            input: input,
            stack: Stack::new(),
        }
    }

    pub fn push_number(&mut self, num: f64) {
        self.stack.push(Token::Number(num));
    }

    pub fn tokens(&self) -> &Vec<Token> {
        &self.stack.data
    }

    pub fn set_data(&mut self, data: Vec<Token>) {
        self.stack.data = data;
    }

    pub fn input(&self) -> String {
        self.input.clone()
    }

    pub fn eval_with_var(&self) -> Option<f64> {
        // substitute variables
        let inp = self.input.clone();

        let binding = Parser::new(inp);
        let tokens = binding.tokens();
        let result = Parser::shunting_yard(tokens.unwrap().1);
        let result = Parser::calculate(result.unwrap());

        if let Ok(num) = result {
            Some(num)
        } else {
            None
        }
    }
}

#[cfg(test)]

mod test_eval_with_var {
    use super::*;

    #[test]
    fn five_x() {
        let expr = Expression::new("5x".to_string());
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), 2.0);
        assert_eq!(expr.eval_with_var().unwrap(), 10.0);
    }

    #[test]
    fn five_sin_x() {
        let expr = Expression::new("5 * sin ( x )".to_string());
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), 2.0);
        assert_eq!(expr.eval_with_var().unwrap().round(), 5.0);
    }

    #[test]
    fn not_a_math_expression() {
        let expr = Expression::new("not an exp".to_string());
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), 2.0);
        let result = expr.eval_with_var();
        assert_eq!(result, None);
    }

    #[test]
    // 2(x+1)/2

    fn complex_expression() {
        let expr = Expression::new("2 ( x + 1 ) / 2".to_string());
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), -1.0);
        assert_eq!(expr.eval_with_var().unwrap(), 0.0);
    }

    #[test]
    fn parse_5_x() {
        let mut p = Parser::new("5x".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        let result = Parser::calculate(result.unwrap());
        assert_eq!(result.unwrap(), 5.0);
    }
}
