
mod parser;


pub use parser::{parse_file, parse_interactive};

pub struct File {
    pub statements: Vec<Statement>,
}

pub struct Interactive {
    pub statement: Statement,
}


pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
}


pub struct Assignment {
    pub target: String,
    pub value: Expression,
}

pub enum Expression {
    Literal(Literal),
    Pipeline(Pipeline),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Literal>),
    Map(Vec<(String, Literal)>),
    Null,
}


pub struct Pipeline {
    pub commands: Vec<Command>,
    pub operators: Vec<Operator>,
}

pub struct Command {
    pub name: String,
    pub arguments: Vec<Literal>,
}

pub enum Operator {
    Pipe,
    And,
    Or,
    Then,
}
