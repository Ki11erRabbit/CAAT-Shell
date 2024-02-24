use crate::{parser::{Expression, File, Pipeline, Statement}, shell::Shell};
use std::io::Write;
use caat_rust::{Caat,Value};
use regex::Regex;
use std::sync::Arc;




pub fn repl(shell: &mut Shell) {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let mut interactive = match crate::parser::parse_interactive(&input) {
            Ok(i) => i,
            Err(msg) => {
                println!("{}", msg);
                continue;
            }
        };
        //eprintln!("{:?}", interactive);
        match eval(shell, &mut interactive) {
            Ok((true, value)) => {
                println!("{}", format_value(&value));
            }
            Ok((false, value)) => {
                println!("{}", format_value(&value));
                break;
            }
            Err(msg) => println!("{}", msg),
        }
    }
}


pub fn run_file(shell: &mut Shell, file: &mut File) -> Value {
    loop {
        match eval(shell, file) {
            Ok((true, value)) => {
                return value;
            }
            Ok((false, value)) => {
                return value;
            }
            Err(msg) => {
                return Value::Failure(msg);
            }
        }
    }
}



fn eval(shell: &mut Shell, input: &mut dyn Iterator<Item = Statement>) -> Result<(bool, Value), String> {
    match input.next() {
        Some(Statement::Assignment(assignment)) => {
            //println!("Assignment: {:?} = {:?}", assignment.target, assignment.value);
            let value = eval_expression(shell, assignment.value)?;
            let env = shell.environment_mut();
            env.set(assignment.target, value);
        }
        Some(Statement::Expression(expression)) => {
            //println!("Expression: {:?}", expression);
            let value = eval_expression(shell, expression)?;
            return Ok((true, value));
        }
        Some(Statement::FunctionDef(function)) => {
            let name = function.name.clone();
            let function = crate::shell::function::Function::new(function.name, function.args, function.body);
            shell.set_function(name, function);
        }
        Some(Statement::Return(expression)) => {
            let value = eval_expression(shell, expression)?;
            return Ok((false, value));
        }
        None => {
            return Ok((false, Value::Null))
        }
    }
    Ok((true, Value::Null))
}




fn eval_expression(shell: &mut Shell, expression: Expression) -> Result<Value,String> {
    match expression {
        Expression::Literal(literal) => {
            Ok(literal.as_value())
        }
        Expression::Pipeline(pipeline) => {
            //println!("Pipeline: {:?}", pipeline);
            eval_pipeline(shell, &pipeline, None)
        }
        Expression::Variable(variable) => {
            let env = shell.environment();
            Ok(env.get(&variable).unwrap().clone())
        }
        Expression::Parenthesized(expression) => {
            eval_expression(shell, *expression)
        }
        Expression::HigherOrder(mut hocmd) => {
            hocmd.resolve_args(shell);
            Ok(Value::CAATFunction(Arc::new(hocmd)))
        }
        Expression::If(cond, then, else_) => {
            match eval_expression(shell, *cond)? {
                Value::Boolean(true) => eval_expression(shell, *then),
                Value::Boolean(false) => eval_expression(shell, *else_),
                _ => Err("if: type error boolean not found".to_string()),
            }
        }
        Expression::Access(thing, index) => {
            let thing = eval_expression(shell, *thing)?;
            let index = eval_expression(shell, *index)?;
            match thing {
                Value::List(list) => {
                    match index {
                        Value::Integer(i) => {
                            Ok(list[i as usize].clone())
                        }
                        _ => Err("List index must be an integer".to_string()),
                    }
                }
                Value::Map(map, _) => {
                    match index {
                        Value::String(s) => {
                            Ok(map.get(&s).unwrap().clone())
                        }
                        _ => Err("Map index must be a string".to_string()),
                    }
                }
                _ => Err("access: type error".to_string()),
            }
        }
    }
}

fn eval_pipeline(shell: &mut Shell, pipeline: &Pipeline, arg: Option<Value>) -> Result<Value, String> {



    let command = &pipeline.command;
    let name = &command.name;
    let args: Vec<Value> = command.arguments_as_value(shell.environment());
    let args = match arg {
        Some(arg) => {
            let mut args = args;
            args.push(arg);
            args
        }
        None => args,
    };

    let value = if let Some(function) = shell.get_function(name) {
        match function.call(&args) {
            Value::Failure(msg) => Err(msg),
            value => Ok(value),
        }
    } else {
        let value = match crate::builtins::run_builtin(Some(shell), name.as_str(), &args) {
            Ok(value) => Ok(value),
            Err(Ok(())) => {
                let ff = caat_rust::ForeignFunction::new(&command.name);
                //println!("{:?}", command.arguments_as_value(shell.environment()));
                let return_value = match ff.call(&command.arguments_as_value(shell.environment())) {
                    Value::Failure(msg) => return Err(msg),
                    value => value,
                };
                Ok(return_value)
            }
            Err(Err(msg)) => Err(msg),
        };
        value
    };

    
    match (&pipeline.operator, &pipeline.next) {
        (Some(crate::parser::Operator::Pipe), Some(next)) => {
            eval_pipeline(shell, next, Some(value?))
        }
        (Some(crate::parser::Operator::Then), Some(next)) => {
            eval_pipeline(shell, next, None) 
        }
        (Some(crate::parser::Operator::And), Some(next)) => {
            match value? {
                Value::Failure(msg) => return Err(msg),
                _ => {}
            }
            eval_pipeline(shell, next, None) 
        }
        (Some(crate::parser::Operator::Or), Some(next)) => {
            eval_pipeline(shell, next, None) 
        }
        _ => value,
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
                    Value::Map(_, _) => {
                        current += 1;
                        result.push(Vec::new());
                        result[current].extend_from_slice(&format_map(value));
                    }
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
                    output.push_str(&String::from(" ").repeat(longest[i] - cell.chars().count()));
                    output.push_str(cell);
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
        Value::Map(_,_) => format_map(value).join("  "),
        Value::Failure(msg) => format!("Failure: {}", msg),
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

fn format_map(value: &Value) -> Vec<String> {
    match value {
        Value::Map(map, fmt) => {
            match fmt {
                Some(fmt) => {
                    let mut result = Vec::new();
                    let re = Regex::new(r"(\{\S+\})").unwrap();
                    for x in re.captures_iter(fmt.as_str()) {
                        if let Some(x) = x.get(0) {
                            let key = x.as_str().chars().skip(1).take(x.as_str().chars().count() - 2).collect::<String>();
                            
                            if let Some(value) = map.get(&key) {
                                result.push(format!("{}", &format_value(value)));
                            } else {
                                result.push(format!("{}: <not found>", key));
                            }

                        } else {
                            break;
                        }
                    }
                    result
                }
                None => {
                    let mut result = Vec::new();
                    for (key, value) in map.iter() {
                        result.push(format!("{}: {}  ", key, format_value(value)));
                    }
                    result
                }
            }
        }
        _ => unreachable!()
    }
}

