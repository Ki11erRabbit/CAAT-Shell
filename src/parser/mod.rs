
mod parser;


pub use parser::{parse_file, parse_interactive};

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

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Pipe,
    And,
    Or,
    Then,
}
