// Math expression parser from infix to postfix

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    character::complete::space0 as space,
    combinator::{map, map_res, not, opt, peek},
    error::ParseError,
    multi::many0,
    sequence::{delimited, pair, tuple},
    IResult,
};

type Number = f64;

use std::str::FromStr;

trait Stack<T> {
    fn top(&self) -> Option<T>;
}

impl<T: Clone> Stack<T> for Vec<T> {
    fn top(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self[self.len() - 1].clone())
        }
    }
}
#[derive(Debug, Copy, PartialEq, Clone)]
pub struct Operator {
    symbol: char,
    precedence: u8,
    is_left_associative: bool,
    operation: fn(f64, f64) -> f64,
}

impl Operator {
    pub fn new(
        symbol: &str,
        precedence: u8,
        is_left_associative: bool,
        operation: fn(Number, Number) -> Number,
    ) -> Token {
        Token::Operator(Operator {
            symbol: symbol.to_string().chars().next().unwrap(),
            precedence,
            is_left_associative,
            operation,
        })
    }

    fn apply(&self, a: Number, b: Number) -> Number {
        (self.operation)(a, b)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Token can mean a number, word, operator
    Number(f64),
    Variable(Variable),
    Operator(Operator),
    LeftParen,
    RightParen,
    Negate,

    Function(Function),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Function {
    Sin,
    Cos,
    Tan,
}

impl Token {
    pub fn is_operator(&self) -> bool {
        match self {
            Self::Operator(_) => true,
            _ => false,
        }
    }

    pub fn is_variable(&self) -> bool {
        match self {
            Self::Variable(_) => true,
            _ => false,
        }
    }

    pub fn operator(&self) -> Option<&Operator> {
        match self {
            Self::Operator(op) => Some(op),
            _ => None,
        }
    }
}

trait Stringify {
    fn to_string(&self) -> String;
}

impl Stringify for Vec<Token> {
    fn to_string(&self) -> String {
        self.iter()
            .map(|t| match t {
                Token::Number(n) => n.to_string(),
                Token::Variable(v) => match v {
                    Variable::X => "x".to_string(),
                    Variable::Y => "y".to_string(),
                    Variable::Z => "z".to_string(),
                },
                Token::Operator(op) => op.symbol.to_string(),
                Token::LeftParen => "(".to_string(),
                Token::RightParen => ")".to_string(),
                Token::Negate => "-".to_string(),
                Token::Function(f) => match f {
                    Function::Sin => "sin".to_string(),
                    Function::Cos => "cos".to_string(),
                    Function::Tan => "tan".to_string(),
                },
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}
trait Pop {
    fn pop(&mut self) -> Option<Token>;
}

impl Pop for Vec<Token> {
    fn pop(&mut self) -> Option<Token> {
        // pop the last element of the stack
        self.pop()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Variable {
    // allow only x, y, z for now
    X,
    Y,
    Z,
}

pub struct Parser {
    input: String,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            input: "".to_string(),
        }
    }
}

impl Parser {
    pub fn new(i: String) -> Self {
        Self {
            input: i
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(""),
        }
    }

    pub fn parse_number(input: &str) -> IResult<&str, Token> {
        // number could be a float or an integer
        map(
            take_while1(|c: char| c.is_digit(10) || c == '.'),
            |s: &str| Token::Number(f64::from_str(s).unwrap()),
        )(input)
    }

    pub fn parse_variable(input: &str) -> IResult<&str, Token> {
        // variable could be x, y, z
        map(alt((tag("x"), tag("y"), tag("z"))), |s: &str| match s {
            "x" => Token::Variable(Variable::X),
            "y" => Token::Variable(Variable::Y),
            "z" => Token::Variable(Variable::Z),
            _ => panic!("Invalid variable"),
        })(input)
    }

    pub fn parse_function(input: &str) -> IResult<&str, Token> {
        // function could be sin, cos, tan
        map(
            alt((tag("sin"), tag("cos"), tag("tan"))),
            |s: &str| match s {
                "sin" => Token::Function(Function::Sin),
                "cos" => Token::Function(Function::Cos),
                "tan" => Token::Function(Function::Tan),
                _ => panic!("Invalid function"),
            },
        )(input)
    }

    fn negate(input: &str) -> IResult<&str, ()> {
        map(tuple((opt(tag(" ")), char('-'), opt(tag(" ")))), |_| ())(input)
    }
    fn parse_operator(input: &str) -> IResult<&str, Token> {
        let (input, symbol) = alt((
            tag("+"),
            tag("-"),
            tag("*"),
            tag("/"),
            tag("^"),
            tag("("),
            tag(")"),
        ))(input)?;
        let op = match symbol {
            "+" => Operator::new("+", 2, true, |a, b| a + b),
            "-" => {
                let (input, _) = peek(not(tag("-")))(input)?;
                return Ok((input, Token::Negate));
            }
            "*" => Operator::new("*", 3, true, |a, b| a * b),
            "/" => Operator::new("/", 3, true, |a, b| a / b),
            "^" => Operator::new("^", 4, false, |a, b| a.powf(b)),
            "(" => Token::LeftParen,
            ")" => Token::RightParen,
            _ => unreachable!(),
        };
        Ok((input, op))
    }

    fn token(input: &str) -> IResult<&str, Token> {
        // parser to match all tokens, including space and the operators
        alt((
            Self::parse_number,
            Self::parse_variable,
            Self::parse_operator,
            Self::parse_function,
        ))(input)
    }

    pub fn tokens(&self) -> IResult<&str, Vec<Token>> {
        // parse a string into tokens
        many0(Self::token)(self.input.as_str())
    }

    fn tilt_until(operators: &mut Vec<Token>, output: &mut Vec<Token>, stop: Token) -> bool {
        while let Some(token) = operators.pop() {
            if token == stop {
                return true;
            }
            output.push(token)
        }
        false
    }

    pub fn shunting_yard(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
        let mut output: Vec<Token> = Vec::new();
        let mut operators: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(_) => output.push(token),
                Token::LeftParen => operators.push(token),
                Token::Function(_) => operators.push(token),
                Token::Variable(_) => output.push(token),
                Token::Negate => operators.push(token),
                Token::Operator(operator) => {
                    while let Some(top) = operators.top() {
                        match top {
                            Token::LeftParen => break,
                            Token::Operator(top_op) => {
                                let p = top_op.precedence;
                                let q = operator.precedence;

                                if (top_op.is_left_associative && p >= q)
                                    || (!top_op.is_left_associative && p > q)
                                {
                                    output.push(operators.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            Token::Number(_) => todo!(),
                            Token::Variable(_) => todo!(),
                            Token::RightParen => {
                                while (top != Token::LeftParen) {
                                    assert!(operators.pop().is_some());
                                }
                            }
                            Token::Negate => todo!(),
                            Token::Function(_) => todo!(),
                        }
                    }
                    operators.push(token);
                }
                Token::RightParen => {
                    if !Self::tilt_until(&mut operators, &mut output, Token::LeftParen) {
                        return Err(String::from("Mismatched ')'"));
                    }
                }
            }
        }

        if Self::tilt_until(&mut operators, &mut output, Token::LeftParen) {
            return Err(String::from("Mismatched '('"));
        }

        assert!(operators.is_empty());
        Ok(output)
    }

    pub fn calculate(postfix_tokens: Vec<Token>) -> Result<Number, String> {
        let mut stack = Vec::new();

        for token in postfix_tokens {
            match token {
                Token::Number(number) => stack.push(number),
                Token::Function(_) => match token {
                    Token::Function(Function::Sin) => {
                        if let Some(x) = stack.pop() {
                            stack.push(x.sin());
                        }
                    }
                    Token::Function(Function::Cos) => {
                        if let Some(x) = stack.pop() {
                            stack.push(x.cos());
                        }
                    }
                    Token::Function(Function::Tan) => {
                        if let Some(x) = stack.pop() {
                            stack.push(x.tan());
                        }
                    }
                    _ => unreachable!("Unexpected function {:?} during calculation", token),
                },
                Token::Operator(operator) => {
                    if let Some(y) = stack.pop() {
                        if let Some(x) = stack.pop() {
                            stack.push(operator.apply(x, y));
                            continue;
                        }
                    }
                    return Err(format!(
                        "Missing operand for operator '{}'",
                        operator.symbol
                    ));
                }

                Token::Negate => {
                    if let Some(x) = stack.pop() {
                        stack.push(-x);
                    }
                }

                Token::Variable(_) => match token {
                    Token::Variable(Variable::X) => {
                        if let Some(x) = stack.pop() {
                            stack.push(x);
                        }
                    }
                    Token::Variable(Variable::Y) => {
                        if let Some(y) = stack.pop() {
                            stack.push(y);
                        }
                    }
                    Token::Variable(Variable::Z) => {
                        if let Some(z) = stack.pop() {
                            stack.push(z);
                        }
                    }
                    _ => unreachable!("Unexpected variable {:?} during calculation", token),
                },
                _ => unreachable!("Unexpected token {:?} during calculation", token),
            }
        }

        if stack.len() != 1 {
            return Err(format!("Expected 1 value on stack, found {}", stack.len()));
        }
        Ok(stack.pop().unwrap())
    }
}

#[cfg(test)]

mod test_parser {
    use super::{Function, Operator, Token};
    use crate::parser::{Parser, Variable};

    // evals
    #[test]
    fn test_float() {
        let mut p = Parser::new("1.2".to_string());
        let result = p.tokens();
        assert_eq!(result.unwrap().1, vec![Token::Number(1.2)]);
    }

    #[test]
    fn test_int() {
        let mut p = Parser::new("1".to_string());
        let result = p.tokens();
        assert_eq!(result.unwrap().1, vec![Token::Number(1.0)]);
    }

    #[test]
    fn test_variable() {
        let mut p = Parser::new("x".to_string());
        let result = p.tokens();
        assert_eq!(result.unwrap().1, vec![Token::Variable(Variable::X)]);
    }

    #[test]
    fn test_trig() {
        let mut p = Parser::new("sin(x)".to_string());
        let result = p.tokens();
        assert_eq!(
            result.unwrap().1,
            vec![
                Token::Function(Function::Sin),
                Token::LeftParen,
                Token::Variable(Variable::X),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn print_shunting_yard() {
        let mut p = Parser::new("1 + 2 * 3".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        println!("{:?}", result);
    }

    #[test]
    fn test_calculate_4_plus_2_times_3() {
        let mut p = Parser::new("4 + 2 * 3".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        let result = Parser::calculate(result.unwrap());
        assert_eq!(result.unwrap(), 10.0);
    }

    #[test]
    fn test_calculate_4_dot_5_plus_2_times_3() {
        let mut p = Parser::new("4.5 + 2 * 3".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        let result = Parser::calculate(result.unwrap());
        assert_eq!(result.unwrap(), 10.5);
    }

    #[test]
    fn test_calculate_4_leftparen_2_plus_3_rightparen() {
        let mut p = Parser::new("4 * (2 + 3)".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        let result = Parser::calculate(result.unwrap());
        assert_eq!(result.unwrap(), 20.0);
    }

    #[test]
    fn test_negative_number() {
        let mut p = Parser::new("-1".to_string());
        let result = p.tokens();
        let result = Parser::shunting_yard(result.unwrap().1);
        println!("{:?}", result);
        let result = Parser::calculate(result.unwrap());
        assert_eq!(result.unwrap(), -1.0);
    }
}
