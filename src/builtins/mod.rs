use caat_rust::Value;

use crate::{parser::Command, shell::Shell};

mod echo;
mod cd;
mod ls;
mod map;




pub fn run_builtin(shell: &mut Shell, command: &Command) -> Result<Value,Result<(),String>> {
    let name = &command.name;
    let output = match name.as_str() {
        "echo" => echo::echo(command.arguments_as_value(shell.environment())), 
        "cd" => cd::cd(command.arguments_as_value(shell.environment())).map_err(|msg| Err(msg))?,
        "ls" => ls::ls(command.arguments_as_value(shell.environment())).map_err(|msg| Err(msg))?,
        _ => return Err(Ok(())),
    };
    return Ok(output);
}
