use caat_rust::Value;
use crate::shell::Shell;


pub fn background(shell: Option<&mut Shell>, args: &Vec<Value>) -> Result<Value,String> {
    if let Some(shell) = shell {
        if let Some(command) = args.get(0) {
            return shell.job_manager_mut().spawn_command(command.clone(), &args[1..].to_vec());
        } else {
            return Err("background: No command provided".to_string());
        }
    } else {
        return Err("background: Called from bad context".to_string());
    }
}


pub fn join(shell: Option<&mut Shell>, args: &Vec<Value>) -> Result<Value,String> {
    if let Some(shell) = shell {
        if args.len() != 1 {
            return Err("join: Expected 1 argument".to_string());
        }
        return shell.job_manager_mut().join(args[0].clone());
    } else {
        return Err("join: Called from bad context".to_string());
    }
}

pub fn jobs(shell: Option<&mut Shell>, args: &Vec<Value>) -> Result<Value,String> {
    if let Some(shell) = shell {
        return Ok(shell.job_manager().jobs(args));
    } else {
        return Err("jobs: Called from bad context".to_string());
    }
}
