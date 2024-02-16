
mod parser;


use caat_rust::Value;
pub use parser::{parse_file, parse_interactive};

use crate::shell::Environment;

pub struct File {
    pub statements: Vec<Statement>,
}

impl Iterator for File {
    type Item = Statement;
    fn next(&mut self) -> Option<Self::Item> {
        self.statements.pop()
    }
}

pub struct Interactive {
    pub statement: Statement,
    gave_statement: bool,
}

impl Iterator for Interactive {
    type Item = Statement;
    fn next(&mut self) -> Option<Self::Item> {
        if self.gave_statement {
            None
        } else {
            self.gave_statement = true;
            Some(self.statement.clone())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
}


#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub target: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Pipeline(Pipeline),
    Variable(String),
    Parenthesized(Box<Expression>),
}

impl Expression {
    pub fn as_value(&self, env: &Environment) -> Value {
        match self {
            Expression::Literal(literal) => literal.as_value(),
            Expression::Variable(string) => env.get(&string).unwrap().clone(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Literal {
    Integer(i64),
    Float(String),
    String(String),
    Boolean(bool),
    List(Vec<Literal>),
    Map(Vec<(String, Literal)>),
    Null,
}

impl Literal {
    pub fn as_value(&self) -> Value {
        match self {
            Literal::Integer(i) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(f.parse().unwrap()),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::List(l) => Value::List(l.iter().map(|l| l.as_value()).collect()),
            Literal::Map(m) => Value::Map(m.iter().map(|(k, v)| (k.clone(), v.as_value())).collect()),
            Literal::Null => Value::Null,
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Pipeline {
    pub command: Command,
    pub operator: Option<Operator>,
    pub next: Option<Box<Pipeline>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
    pub name: String,
    pub arguments: Vec<Expression>,
}

impl Command {
    pub fn arguments_as_value(&self, env: &Environment) -> Vec<Value> {
        self.arguments.iter().map(|arg| arg.as_value(env)).collect()
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Operator {
    Pipe,
    And,
    Or,
    Then,
}
