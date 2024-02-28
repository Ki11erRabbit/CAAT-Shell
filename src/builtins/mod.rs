use caat_rust::Value;
use crate::shell::Shell;
use std::sync::{Arc, RwLock};

mod echo;
mod cd;
mod ls;
mod background;
mod list_utils;
mod search;
mod numbers;
mod strings;



pub fn run_builtin(shell: Option<Arc<RwLock<Shell>>>, command_name: &str, args: &Vec<Value>) -> Result<Value,Result<(),String>> {
    let output = match command_name {
        "args" => get_args(),
        "trace" => echo::trace(args),
        "echo" => echo::echo(args), 
        "cd" => cd::cd(args).map_err(|msg| Err(msg))?,
        "ls" => ls::ls(args).map_err(|msg| Err(msg))?,
        //"if" => conditionals::if_command(args).map_err(|msg| Err(msg))?,
        "background" => background::background(shell, args).map_err(|msg| Err(msg))?,
        "join" => background::join(shell, args).map_err(|msg| Err(msg))?,
        "jobs" => background::jobs(shell, args).map_err(|msg| Err(msg))?,
        "map" => list_utils::map(args).map_err(|msg| Err(msg))?,
        "fold" => list_utils::fold(args).map_err(|msg| Err(msg))?,
        "filter" => list_utils::filter(args).map_err(|msg| Err(msg))?,
        "concat" => list_utils::concat(args).map_err(|msg| Err(msg))?,
        "shuf" => list_utils::shuf(args).map_err(|msg| Err(msg))?,
        "head" => list_utils::head(args).map_err(|msg| Err(msg))?,
        "tail" => list_utils::tail(args).map_err(|msg| Err(msg))?,
        "rest" => list_utils::rest(args).map_err(|msg| Err(msg))?,
        "find" => search::find(args).map_err(|msg| Err(msg))?,
        "add" => numbers::add(args).map_err(|msg| Err(msg))?,
        "sub" => numbers::sub(args).map_err(|msg| Err(msg))?,
        "mul" => numbers::mult(args).map_err(|msg| Err(msg))?,
        "div" => numbers::div(args).map_err(|msg| Err(msg))?,
        "contains" => strings::contains(args).map_err(|msg| Err(msg))?,
        "split" => strings::split(args).map_err(|msg| Err(msg))?,
        _ => return Err(Ok(())),
    };
    return Ok(output);
}

fn get_args() -> Value {
    let args = caat_rust::args();
    let mut output = Vec::new();
    for arg in args {
        output.push(arg);
    }
    return Value::List(output.into());
}
