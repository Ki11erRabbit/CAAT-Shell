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

        let mut interactive = crate::parser::parse_interactive(&input).unwrap();
        eval(shell, &mut interactive);
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
            println!("{}", format_value(&value));
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

    //println!("Command: {:?}", pipeline);
    let command = &pipeline.command;

    match crate::builtins::run_builtin(shell, command) {
        Ok(value) => value,
        Err(()) => {
            let ff = caat_rust::ForeignFunction::new(&command.name);
            //println!("{:?}", command.arguments_as_value(shell.environment()));
            let return_value = ff.call(&command.arguments_as_value(shell.environment()));
            return_value
        }
    }

}


fn format_value(value: &Value) -> String {
    match value {
        Value::Null => format!("()"),
        Value::String(string) => format!("{}", string),
        Value::List(list) => {
            let mut result: Vec<Vec<String>> = vec![Vec::new()];
            let mut current = 0;
            for value in list.iter() {
                
                match value {
                    Value::List(_) => {
                        current += 1;
                        result.push(Vec::new());
                        result[current].extend_from_slice(&format_list(value));
                    },
                    _ => {
                        result[current].push(format_value(value));
                    }
                }
            }
            let mut longest = Vec::new();
            for row in &result {
                for (i, cell) in row.iter().enumerate() {
                    if longest.len() <= i {
                        longest.push(0)
                    }
                    if cell.chars().count() > longest[i] {
                        longest[i] = cell.chars().count();
                    }
                    
                }
            }
            let mut output = String::new();
            for (r, row) in result.iter().enumerate() {
                for (i, cell) in row.iter().enumerate() {
                    output.push_str(cell);
                    output.push_str(&String::from(" ").repeat(longest[i] - cell.chars().count()));
                    if i < row.len() - 1 {
                        output.push_str("  ");
                    }
                }
                if r < result.len() - 1 {
                    output.push_str("\n");
                }
            }
            output
        }
        Value::Float(f) => format!("{}", f),
        Value::Integer(i) => format!("{}", i),
        Value::Boolean(b) => format!("{}", b),
        Value::CAATFunction(_) => format!("<foreign function>"),
        Value::Map(_) => unimplemented!(),
    }
}


fn format_list(value: &Value) -> Vec<String> {
    match value {
        Value::List(list) => {
            let mut result = Vec::new();
            for value in list.iter() {
                
                match value {
                    Value::List(_) => unreachable!(),
                    _ => {}
                }
                result.push(format_value(value));
            }
            result
        }
        _ => unreachable!()
    }
}

