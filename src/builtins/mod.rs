use caat_rust::Value;

use crate::{parser::Command, shell::Shell};

mod echo;
mod cd;
mod ls;





pub fn run_builtin(shell: &mut Shell, command: &Command) -> Result<Value,()> {
    let name = &command.name;
    match name.as_str() {
        "echo" => Ok(echo::echo(command.arguments_as_value(shell.environment()))),
        "cd" => Ok(cd::cd(command.arguments_as_value(shell.environment()))),
        "ls" => Ok(ls::ls(command.arguments_as_value(shell.environment()))),
        _ => Err(()),
    }
}
