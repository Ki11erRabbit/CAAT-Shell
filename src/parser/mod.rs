
//mod parser;

mod peg_parser;

use caat_rust::{Caat, Value};
pub use peg_parser::{parse_file, parse_interactive};
use std::fmt;
use std::sync::Arc;

use crate::shell::{Environment, Shell};

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
    pub statement: Option<Statement>,
}

impl Iterator for Interactive {
    type Item = Statement;
    fn next(&mut self) -> Option<Self::Item> {
        return self.statement.take();
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
    HigherOrder(Pipeline),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn as_value(&self, env: &Environment) -> Value {
        match self {
            Expression::Literal(literal) => literal.as_value(),
            Expression::Variable(string) => env.get(&string).unwrap().clone(),
            Expression::Pipeline(pipeline) => pipeline.call(&[]),
            Expression::Parenthesized(expression) => expression.as_value(env),
            Expression::HigherOrder(ho) => {
                let mut ho = ho.clone();
                ho.resolve_args_env(env);
                Value::CAATFunction(Arc::new(ho))
            },
            Expression::If(cond, then, else_) => {
                let cond = cond.as_value(env);
                if let Value::Boolean(b) = cond {
                    if b {
                        then.as_value(env)
                    } else {
                        else_.as_value(env)
                    }
                } else {
                    Value::Failure(String::from("if: type error, expected boolean"))
                }
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Literal(l) => write!(f, "{}", l),
            Expression::Pipeline(p) => write!(f, "{}", p),
            Expression::Variable(v) => write!(f, "{}", v),
            Expression::Parenthesized(e) => write!(f, "({})", e),
            Expression::HigherOrder(h) => write!(f, "{}", h),
            Expression::If(cond, then, else_) => write!(f, "if {} then {} else {}", cond, then, else_),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
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
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::List(l) => Value::List(l.iter().map(|l| l.as_value()).collect()),
            Literal::Map(m) => Value::Map(m.iter().map(|(k, v)| (k.clone(), v.as_value())).collect()),
            Literal::Null => Value::Null,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::List(l) => {
                write!(f, "[")?;
                for (i, item) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Literal::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Literal::Null => write!(f, "()"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pipeline {
    pub command: Command,
    pub operator: Option<Operator>,
    pub next: Option<Box<Pipeline>>,
}

impl Pipeline {
    pub fn new(command: Command) -> Pipeline {
        Pipeline {
            command,
            operator: None,
            next: None,
        }
    }

    pub fn resolve_args(&mut self, shell: &Shell) {
        self.command.resolve_args(shell);
        if let Some(next) = &mut self.next {
            next.resolve_args(shell);
        }
    }
    pub fn resolve_args_env(&mut self, env: &Environment) {
        self.command.resolve_args_env(env);
        if let Some(next) = &mut self.next {
            next.resolve_args_env(env);
        }
    }
}

impl Caat for Pipeline {
    fn call(&self, args: &[Value]) -> Value {
        let ff = caat_rust::ForeignFunction::new(&self.command.name);
        let mut new_args = self.command.args.clone();
        new_args.extend_from_slice(args);
        match crate::builtins::run_builtin(None, self.command.name.as_str(), &new_args) {
            Ok(value) => {
                match (&self.operator, &self.next) {
                    (Some(Operator::Pipe), Some(next)) => {
                        return next.call(&[value]);
                    }
                    (Some(Operator::Then), Some(next)) => {
                        return next.call(&[]);
                    }
                    (Some(Operator::And), Some(next)) => {
                        return next.call(&[]);
                    }
                    (Some(Operator::Or), Some(next)) => {
                        return next.call(&[]);
                    }
                    _ => value,
                }
            },
            Err(Ok(())) => {
                let value = ff.call(&new_args);
                match (&self.operator, &self.next) {
                    (Some(Operator::Pipe), Some(next)) => {
                        return next.call(&[value]);
                    }
                    (Some(Operator::Then), Some(next)) => {
                        return next.call(&[]);
                    }
                    (Some(Operator::And), Some(next)) => {
                        return next.call(&[]);
                    }
                    (Some(Operator::Or), Some(next)) => {
                        return next.call(&[]);
                    }
                    _ => value,
                }
            },
            Err(Err(msg)) => {
                return Value::String(msg);
            }
        }
    }
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.command.name)?;
        for arg in &self.command.arguments {
            write!(f, " {}", arg)?;
        }
        if let Some(next) = &self.next {
            write!(f, " | {}", next)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub args: Vec<Value>,
}

impl Command {
    pub fn new(name: String, arguments: Vec<Expression>) -> Command {
        Command {
            name,
            arguments,
            args: Vec::new(),
        }
    }
    pub fn arguments_as_value(&self, env: &Environment) -> Vec<Value> {
        self.arguments.iter().map(|arg| arg.as_value(env)).collect()
    }
    
    pub fn resolve_args(&mut self, shell: &Shell) {
        let env = shell.environment();
        self.args = self.arguments_as_value(env);
    }

    pub fn resolve_args_env(&mut self, env: &Environment) {
        self.args = self.arguments_as_value(env);
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Operator {
    Pipe,
    And,
    Or,
    Then,
}


