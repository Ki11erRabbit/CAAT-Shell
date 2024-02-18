use caat_rust::Value;
use crate::shell::Shell;

mod echo;
mod cd;
mod ls;
mod map;
mod conditionals;
mod background;



pub fn run_builtin(shell: Option<&mut Shell>, command_name: &str, args: &Vec<Value>) -> Result<Value,Result<(),String>> {
    let output = match command_name {
        "echo" => echo::echo(args), 
        "cd" => cd::cd(args).map_err(|msg| Err(msg))?,
        "ls" => ls::ls(args).map_err(|msg| Err(msg))?,
        "map" => map::map(args).map_err(|msg| Err(msg))?,
        "if" => conditionals::if_command(args).map_err(|msg| Err(msg))?,
        "background" => background::background(shell, args).map_err(|msg| Err(msg))?,
        "join" => background::join(shell, args).map_err(|msg| Err(msg))?,
        "jobs" => background::jobs(shell, args).map_err(|msg| Err(msg))?,
        _ => return Err(Ok(())),
    };
    return Ok(output);
}
