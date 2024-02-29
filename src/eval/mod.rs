use crate::{parser::{Expression, File, PipelinePart, Redirect, Statement, MatchArm}, shell::Shell};
use crate::{borrow_mut, borrow};
use std::io::Write;
use caat_rust::{Caat, Value};
use regex::Regex;
use std::sync::{Arc, RwLock};
use rustyline::{self,Editor, history, config, error::ReadlineError};


fn create_rustyline() -> Editor<(),history::DefaultHistory> {
    let config = config::Builder::new()
        .behavior(config::Behavior::PreferTerm)
        .auto_add_history(true)
        .bell_style(config::BellStyle::Audible)
        .completion_type(config::CompletionType::List)
        .build();
    let readline = Editor::with_config(config).unwrap();
    readline
}



pub fn repl(shell: Arc<RwLock<Shell>>) {
    let mut readline = create_rustyline();
    loop {
        match readline.readline("> ") {
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
            Ok(line) => {
                let input = line.trim().to_string();

                let mut interactive = match crate::parser::parse_interactive(&input) {
                    Ok(i) => i,
                    Err(msg) => {
                        println!("{}", msg);
                        continue;
                    }
                };
                //eprintln!("{:?}", interactive);
                match eval(shell.clone(), &mut interactive) {
                    Ok(EvalContext {should_return: false, value, ..}) => {
                        println!("{}", format_value(&value));
                    }
                    Ok(EvalContext {should_return: true, value, ..}) => {
                        println!("{}", format_value(&value));
                        break;
                    }
                    Err(msg) => println!("{}", msg),
                }

            }
        }
        /*print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();*/
    }
}

/*fn parse_file(shell: Arc<RwLock<Shell>>, file_path: &str) -> Result<Value, String> {
    let file = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    match crate::parser::parse_shebang(&file) {
        Ok(shebang) => {
            //eprintln!("shebang: {}", shebang);
                let mut command = std::process::Command::new(shebang);
                command.arg(file_path);
                let status = command.status().map_err(|e| e.to_string())?;
                return Ok(Value::Integer(status.code().unwrap() as i64))
                        
                        
        }
        Err(e) => {
            eprintln!("error: {}", e);
        }
    }

    //eprintln!("file: {}", file);
    let mut file = crate::parser::parse_file(&file).map_err(|e| e.to_string())?;

    Ok(run_file(shell, &mut file))
}*/

pub fn run_file(shell: Arc<RwLock<Shell>>, file: &mut File) -> EvalContext {
    loop {
        match eval(shell.clone(), file) {
            Ok(EvalContext {should_return: false, ..}) => {
                //TODO: add code that enables and disables this
                //println!("{}", format_value(&value));
            }
            Ok(ctx) => {
                return ctx;
            }
            Err(msg) => {
                println!("{}", msg);
                return EvalContext::new(Value::Failure(msg));
            }
        }
    }
}

pub struct EvalContext {
    value: Value,
    should_return: bool,
    loop_state: LoopState,
}

impl EvalContext {
    fn new(value: Value) -> Self {
        EvalContext {
            value,
            should_return: false,
            loop_state: LoopState::None,
        }
    }
    
    fn new_should_return(value: Value, should_return: bool) -> Self {
        EvalContext {
            value,
            should_return,
            loop_state: LoopState::None,
        }
    }

    fn new_loop_state(value: Value, loop_state: LoopState) -> Self {
        EvalContext {
            value,
            should_return: false,
            loop_state,
        }
    }
    pub fn get_value(self) -> Value {
        self.value
    }
}

pub enum LoopState {
    Continue,
    Break,
    None,
}

fn eval(shell: Arc<RwLock<Shell>>, input: &mut dyn Iterator<Item = Statement>) -> Result<EvalContext, String> {
    let next = input.next();
    match next {
        Some(Statement::Assignment(assignment)) => {
            let value = eval_expression(shell.clone(), assignment.value)?;
            //eprintln!("Assignment: {:?} = {:?}", assignment.target, value);
            let mut borrowed_shell = borrow_mut!(shell);
            let env = borrowed_shell.environment_mut();
            env.set(assignment.target, value);
        }
        Some(Statement::Expression(expression)) => {
            //println!("Expression: {:?}", expression);
            let value = eval_expression(shell, expression)?;
            return Ok(EvalContext::new(value));
        }
        Some(Statement::FunctionDef(function)) => {
            let name = function.name.clone();
            let function = crate::shell::function::Function::new(&function.name, function.args, function.body, shell.clone());
            let mut borrowed_shell = borrow_mut!(shell);
            borrowed_shell.set_function(name, function);
        }
        Some(Statement::Return(expression)) => {
            let value = eval_expression(shell, expression)?;
            return Ok(EvalContext::new_should_return(value, true));
        }
        Some(Statement::Blank) => {}
        Some(Statement::Comment(_)) => {}
        Some(Statement::Break) => {
            return Ok(EvalContext::new_loop_state(Value::Null, LoopState::Break));
        }
        Some(Statement::Continue) => {
            return Ok(EvalContext::new_loop_state(Value::Null, LoopState::Continue));
        }
        Some(Statement::Loop(body)) => {
            let mut body = body.peekable();
            let original = body.clone();
            loop {
                if let None = body.peek() {
                    body = original.clone();
                }
                match eval(shell.clone(), &mut body) {
                    Ok(EvalContext {loop_state: LoopState::Continue, ..}) => {
                        continue;
                    }
                    Ok(EvalContext {loop_state: LoopState::Break, ..}) => {
                        break;
                    }
                    Ok(EvalContext {should_return: false, loop_state: LoopState::None, ..}) => {}
                    Ok(ctx) => {
                        return Ok(ctx);
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            }
        }
        None => {
            return Ok(EvalContext::new_should_return(Value::Null, true));
        }
    }
    Ok(EvalContext::new(Value::Null))
}




fn eval_expression(shell: Arc<RwLock<Shell>>, expression: Expression) -> Result<Value,String> {
    match expression {
        Expression::Literal(literal) => {
            Ok(literal.as_value())
        }
        Expression::Pipeline(pipeline) => {
            //println!("Pipeline: {:?}", pipeline);
            let result = eval_pipeline(shell.clone(), &pipeline.pipeline, None)?;
            if let Some(redirect) = pipeline.redirect {
                match redirect {
                    Redirect::Input(_) => unimplemented!(),
                    Redirect::Output(expr) => {
                        let value = eval_expression(shell, *expr)?;
                        match value {
                            Value::String(string) => {
                                let mut file = std::fs::File::create(string).map_err(|e| e.to_string())?;
                                writeln!(file, "{}", format_value_file(&result)).unwrap();
                            }
                            _ => return Err(">: type error".to_string()),
                        }
                        return Ok(Value::Null);
                    },
                    Redirect::Append(expr) => {
                        let value = eval_expression(shell, *expr)?;
                        match value {
                            Value::String(string) => {
                                let mut file = std::fs::OpenOptions::new().append(true).create(true).open(string).map_err(|e| e.to_string())?;
                                writeln!(file, "{}", format_value_file(&result)).unwrap();
                            }
                            _ => return Err(">>: type error".to_string()),
                        }
                        return Ok(Value::Null);
                    },
                }
            }
            
            Ok(result)
        }
        Expression::Variable(variable) => {
            let borrowed_shell = borrow!(shell);
            match borrowed_shell.get_function(&variable) {
                Some(function) => {
                    return Ok(Value::CAATFunction(Arc::new(function)));
                }
                None => {}
            }
            let env = borrowed_shell.environment();
            Ok(env.get(&variable).ok_or(format!("{} not found in environment", variable))?.clone())
        }
        Expression::Parenthesized(expression) => {
            eval_expression(shell, *expression)
        }
        Expression::HigherOrder(mut hocmd) => {
            hocmd.resolve_args(shell.clone());
            Ok(Value::CAATFunction(Arc::new(hocmd.pipeline)))
        }
        Expression::If(cond, then, else_) => {
            match eval_expression(shell.clone(), *cond)? {
                Value::Boolean(true) => eval_expression(shell.clone(), *then),
                Value::Boolean(false) => eval_expression(shell, *else_),
                _ => Err("if: type error boolean not found".to_string()),
            }
        }
        Expression::Access(thing, index) => {
            let thing = eval_expression(shell.clone(), *thing)?;
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
        },
        Expression::Concat(a, b) => {
            let a = eval_expression(shell.clone(), *a)?;
            let b = eval_expression(shell, *b)?;
            match (a, b) {
                (Value::List(a), Value::List(b)) => {
                    let mut a = a.to_vec();
                    a.extend_from_slice(&b);
                    Ok(Value::List(a.into()))
                }
                (Value::String(mut a), Value::String(b)) => {
                    a.push_str(&b);
                    Ok(Value::String(a))
                }
                _ => Err("concat: type error".to_string()),
            }
        },
        Expression::Lambda(args, body) => {
            let mut lambda = crate::shell::function::Function::new("lambda", args, body, shell.clone());
            let borrowed_shell = borrow!(shell);
            let env = borrowed_shell.environment();
            lambda.bind_environment(env.get_current());
            return Ok(Value::CAATFunction(Arc::new(lambda)));
        },
        Expression::Match(expr, arms) => {
            let expr = eval_expression(shell.clone(), *expr)?;
            for arm in arms {
                match arm {
                    MatchArm::WildcardBind(var, body) => {
                        let mut borrowed_shell = borrow_mut!(shell);
                        let env = borrowed_shell.environment_mut();
                        env.set(var.clone(), expr.clone());
                        drop(borrowed_shell);
                        let result = eval_expression(shell.clone(), body);
                        let mut borrowed_shell = borrow_mut!(shell);
                        let env = borrowed_shell.environment_mut();
                        env.remove(&var);
                        return result;
                    },
                    MatchArm::WildcardDiscard(body) => {
                        return eval_expression(shell, body);
                    },
                    MatchArm::Expression(pattern, body) => {
                        let pattern = eval_expression(shell.clone(), pattern)?;
                        if pattern == expr {
                            return eval_expression(shell, body);
                        }
                    },
                }
            }
            return Err("match: no match".to_string());
        }

    }
}

fn eval_pipeline(shell: Arc<RwLock<Shell>>, pipeline: &PipelinePart, arg: Option<Value>) -> Result<Value, String> {



    let command = &pipeline.command;
    let name = &command.name;
    let args: Vec<Value> = command.arguments_as_value(shell.clone());
    let args = match arg {
        Some(arg) => {
            let mut args = args;
            args.push(arg);
            args
        }
        None => args,
    };

    let borrowed_shell = borrow_mut!(shell);
    let value = if let Some(function) = borrowed_shell.get_function(name) {
        drop(borrowed_shell);
        match function.call(&args) {
            Value::Failure(msg) => Err(msg),
            value => Ok(value),
        }
    } else {
        drop(borrowed_shell);
        let value = match crate::builtins::run_builtin(Some(shell.clone()), name.as_str(), &args) {
            Ok(value) => Ok(value),
            Err(Ok(())) => {
                let borrowed_shell = borrow!(shell);
                match borrowed_shell.environment().get(name) {
                    Some(Value::CAATFunction(f)) => {
                        let f = f.clone();
                        drop(borrowed_shell);
                        let value = f.call(&args);
                        match value {
                            Value::Failure(msg) => return Err(msg),
                            value => Ok(value),
                        }
                    },
                    _ => {
                        let ff = caat_rust::ForeignFunction::new(&command.name);
                        //println!("{:?}", command.arguments_as_value(shell.environment()));
                        drop(borrowed_shell);
                        let return_value = match ff.call(&command.arguments_as_value(shell.clone())) {
                            Value::Failure(msg) => {
                                return Err(msg);
                            },
                            value => value,
                        };
                        Ok(return_value)
                    }
                }
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
                        if result[current].len() > 0 {
                            current += 1;
                            result.push(Vec::new());
                        }
                        result[current].extend_from_slice(&format_list(value));
                    },
                    Value::Map(_, _) => {
                        if result[current].len() > 0 {
                            current += 1;
                            result.push(Vec::new());
                        }
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

fn format_value_file(value: &Value) -> String {
    match value {
        Value::Null => format!("()"),
        Value::String(string) => format!("{}", string),
        Value::List(list) => {
            let mut result: Vec<Vec<String>> = vec![Vec::new()];
            let mut current = 0;
            for value in list.iter() {
                
                match value {
                    Value::List(_) => {
                        if result[current].len() > 0 {
                            current += 1;
                            result.push(Vec::new());
                        }
                        result[current].extend_from_slice(&format_list(value));
                    },
                    Value::Map(_, _) => {
                        if result[current].len() > 0 {
                            current += 1;
                            result.push(Vec::new());
                        }
                        result[current].extend_from_slice(&format_map(value));
                    }
                    _ => {
                        if result[current].len() > 0 {
                            current += 1;
                            result.push(Vec::new());
                        }
                        result[current].extend_from_slice(&vec![format_value_file(value)]);
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
                    if i < row.len() - 1 {
                        output.push_str(",");
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
