
//mod parser;

mod peg_parser;

use caat_rust::{Caat, Value};
pub use peg_parser::{parse_file, parse_interactive, parse_shebang};
use std::fmt;
use std::sync::{Arc, RwLock};
use crate::shell::function::Function;
use crate::{borrow, borrow_mut};
use crate::shell::Shell;

#[derive(Debug, PartialEq, Clone)]
pub struct File {
    pub statements: Option<Vec<Statement>>,
}

impl File {
    pub fn new(statements: Vec<Statement>) -> File {
        File {
            statements: Some(statements),
        }
    }
}

impl Iterator for File {
    type Item = Statement;
    fn next(&mut self) -> Option<Self::Item> {
        let statements = self.statements.take();
        let mut iter = statements.unwrap().into_iter();
        let next = iter.next();
        self.statements = Some(iter.collect());
        next
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = Ok(());
        if let Some(statements) = &self.statements {
            for statement in statements {
                result = write!(f, "{}\n", statement);
            }
        }
        result
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    FunctionDef(FunctionDef),
    Return(Expression),
    Comment(String),
    Blank,
    Break,
    Continue,
    Loop(File),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Assignment(a) => write!(f, "{}", a),
            Statement::Expression(e) => write!(f, "{}", e),
            Statement::FunctionDef(fd) => write!(f, "{}", fd),
            Statement::Return(e) => write!(f, "return {}", e),
            Statement::Comment(c) => write!(f, "# {}", c),
            Statement::Blank => write!(f, ""),
            Statement::Break => write!(f, "break"),
            Statement::Continue => write!(f, "continue"),
            Statement::Loop(body) => write!(f, "loop {{{}}}", body),
                    
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub target: String,
    pub value: Expression,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<String>,
    pub body: File,
}

impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn {}({}) {}", self.name, self.args.join(", "), self.body)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Pipeline(Pipeline),
    Variable(String),
    Parenthesized(Box<Expression>),
    HigherOrder(Pipeline),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    Access(Box<Expression>, Box<Expression>),
    Concat(Box<Expression>, Box<Expression>),
    Lambda(Vec<String>, File),
    Match(Box<Expression>, Vec<MatchArm>),
}

impl Expression {
    pub fn as_value(&self, shell: Arc<RwLock<Shell>>) -> Value {
        let borrowed_shell = borrow!(shell);
        let env = borrowed_shell.environment();
        match self {
            Expression::Literal(literal) => literal.as_value(),
            Expression::Variable(string) => env.get(&string).map_or(Value::Failure(format!("{} not found in environment", string)), |v| v.clone()),
            Expression::Pipeline(pipeline) => pipeline.pipeline.call(&[]),
            Expression::Parenthesized(expression) => {
                drop(borrowed_shell);
                expression.as_value(shell)
            },
            Expression::HigherOrder(ho) => {
                drop(borrowed_shell);
                let mut ho = ho.clone();
                ho.resolve_args(shell.clone());
                Value::CAATFunction(Arc::new(ho.pipeline))
            },
            Expression::If(cond, then, else_) => {
                drop(borrowed_shell);
                let cond = cond.as_value(shell.clone());
                if let Value::Boolean(b) = cond {
                    if b {
                        then.as_value(shell)
                    } else {
                        else_.as_value(shell)
                    }
                } else {
                    Value::Failure(String::from("if: type error, expected boolean"))
                }
            },
            Expression::Access(thing, index) => {
                drop(borrowed_shell);
                let thing = thing.as_value(shell.clone());
                let index = index.as_value(shell.clone());
                match thing {
                    Value::List(list) => {
                        if let Value::Integer(i) = index {
                            if i < 0 || i as usize >= list.len() {
                                return Value::Failure(format!("Index out of bounds: {}", i));
                            }
                            list[i as usize].clone()
                        } else {
                            Value::Failure(String::from("Index must be an integer"))
                        }
                    },
                    Value::Map(map, _) => {
                        if let Value::String(s) = index {
                            if let Some(value) = map.get(&s) {
                                return value.clone();
                            } else {
                                return Value::Failure(format!("Key not found: {}", s));
                            }
                        } else {
                            return Value::Failure(String::from("Key must be a string"));
                        }
                    },
                    _ => Value::Failure(format!("Can't access {} with index {}", thing, index)),
                }
            },
            Expression::Concat(a, b) => {
                drop(borrowed_shell);
                let a = a.as_value(shell.clone());
                let b = b.as_value(shell.clone());
                match (a, b) {
                    (Value::List(a), Value::List(b)) => {
                        let mut a = a.to_vec();
                        a.extend_from_slice(&b);
                        Value::List(a.into())
                    },
                    (Value::String(mut a), Value::String(b)) => {
                        a.push_str(&b);
                        Value::String(a)
                    },
                    (a, b) => Value::Failure(format!("Can't concatenate {} and {}", a, b)),
                }
            }
            Expression::Lambda(args, body) => {
                let mut lambda = Function::new("lambda", args.to_vec(), body.clone(), shell.clone());
                let env = env.get_current();
                lambda.bind_environment(env);
                return Value::CAATFunction(Arc::new(lambda));
            }
            Expression::Match(expr, arms) => {
                drop(borrowed_shell);
                let expr = expr.as_value(shell.clone());
                for arm in arms {
                    match arm {
                        MatchArm::WildcardBind(var, body) => {
                            let mut borrowed_shell = borrow_mut!(shell);
                            let env = borrowed_shell.environment_mut();
                            env.push_scope();
                            env.set(var.clone(), expr.clone());
                            drop(borrowed_shell);
                            let result = body.as_value(shell.clone());
                            let mut borrowed_shell = borrow_mut!(shell);
                            let env = borrowed_shell.environment_mut();
                            env.pop_scope();
                            return result;
                        },
                        MatchArm::WildcardDiscard(body) => {
                            return body.as_value(shell.clone());
                        },
                        MatchArm::Expression(pattern, body) => {
                            let pattern = pattern.as_value(shell.clone());
                            if pattern == expr {
                                return body.as_value(shell.clone());
                            }
                        },
                    }
                }
                return Value::Failure(String::from("No match"));
                    
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
            Expression::Access(thing, index) => write!(f, "{}[{}]", thing, index),
            Expression::Concat(a, b) => write!(f, "{} ++ {}", a, b),
            Expression::Lambda(args, body) => {
                write!(f, "fn ({})", args.join(", "))?;
                write!(f, "{{ {} }}", body)
            }
            Expression::Match(expr, arms) => {
                let out = write!(f, "match {} with", expr);
                for arm in arms {
                    write!(f, "{}", arm)?;
                }
                out
            }
                    
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MatchArm {
    Expression(Expression, Expression),
    WildcardBind(String, Expression),
    WildcardDiscard(Expression),
}

impl fmt::Display for MatchArm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchArm::Expression(e, b) => write!(f, "{} => {}", e, b),
            MatchArm::WildcardBind(s, e) => write!(f, "{} => {}", s, e),
            MatchArm::WildcardDiscard(e) => write!(f, "_ => {}", e),
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
            Literal::Map(m) => {
                let map = m.iter().map(|(k, v)| (k.clone(), v.as_value())).collect();
                Value::Map(map, None)
            },
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
    pub pipeline: PipelinePart,
    pub redirect: Option<Redirect>,
}

impl Pipeline {
    pub fn resolve_args(&mut self, shell: Arc<RwLock<Shell>>) {
        self.pipeline.resolve_args(shell);
    }
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "{}", self.pipeline);
        if let Some(redirect) = &self.redirect {
            write!(f, " {}", redirect)?;
        }
        result
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    Input(Box<Expression>),
    Output(Box<Expression>),
    Append(Box<Expression>),
}

impl fmt::Display for Redirect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Redirect::Input(s) => write!(f, "< {}", s),
            Redirect::Output(s) => write!(f, "> {}", s),
            Redirect::Append(s) => write!(f, ">> {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PipelinePart {
    pub command: Command,
    pub operator: Option<Operator>,
    pub next: Option<Box<PipelinePart>>,
}

impl PipelinePart {
    pub fn new(command: Command) -> PipelinePart {
        PipelinePart {
            command,
            operator: None,
            next: None,
        }
    }

    pub fn resolve_args(&mut self, shell: Arc<RwLock<Shell>>) {
        self.command.resolve_args(shell.clone());
        if let Some(next) = &mut self.next {
            next.resolve_args(shell);
        }
    }
}

impl Caat for PipelinePart {
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

impl fmt::Display for PipelinePart {
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
    pub fn arguments_as_value(&self, shell: Arc<RwLock<Shell>>) -> Vec<Value> {
        self.arguments.iter().map(|arg| arg.as_value(shell.clone())).collect()
    }
    
    pub fn resolve_args(&mut self, shell: Arc<RwLock<Shell>>) {
        self.args = self.arguments_as_value(shell);
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Operator {
    Pipe,
    And,
    Or,
    Then,
}


