use crate::{parser::{Expression, Pipeline, Statement}, shell::Shell};
use std::io::Write;
use caat_rust::Value;






pub fn repl(shell: &mut Shell) {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        //let mut interactive = crate::parser::parse_interactive(&input).unwrap();
        //eval(shell, &mut interactive);
    }
}




fn eval(shell: &mut Shell, input: &mut dyn Iterator<Item = Statement>) {
    match input.next() {
        Some(Statement::Assignment(assignment)) => {
            println!("Assignment: {:?} = {:?}", assignment.target, assignment.value);
            let value = eval_expression(shell, assignment.value);
            let env = shell.environment_mut();
            env.set(assignment.target, value);
        }
        Some(Statement::Expression(expression)) => {
            //println!("Expression: {:?}", expression);
            let value = eval_expression(shell, expression);
            println!("Value: {:?}", value);
        }
        None => {}
    }
}




fn eval_expression(shell: &mut Shell, expression: Expression) -> Value {
    match expression {
        Expression::Literal(literal) => {
            literal.as_value()
        }
        Expression::Pipeline(pipeline) => {
            //println!("Pipeline: {:?}", pipeline);
            eval_pipeline(shell, Box::new(pipeline))
        }
        Expression::Variable(variable) => {
            let env = shell.environment();
            env.get(&variable).unwrap().clone()
        }
        Expression::Parenthesized(expression) => {
            eval_expression(shell, *expression)
        }
            
    }
}

fn eval_pipeline(shell: &mut Shell, mut pipeline: Box<Pipeline>) -> Value {

    println!("Command: {:?}", pipeline);
    let command = &pipeline.command;

    let ff = caat_rust::ForeignFunction::new(&command.name);
    println!("{:?}", command.arguments_as_value(shell.environment()));
    let return_value = ff.call(&command.arguments_as_value(shell.environment()));
    return_value
}
