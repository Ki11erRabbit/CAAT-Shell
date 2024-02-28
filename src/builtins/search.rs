use std::process::Command;

use caat_rust::Value;






pub fn find(args: &Vec<Value>) -> Result<Value,String> {
    let mut command = Command::new("find");
    for arg in args {
        match arg {
            Value::String(s) => command.arg(s),
            _ => {
                return Err("find: expected string arguments".to_string());
            }
        };
    }
    let output = command.output().map_err(|e| format!("find: {}", e))?;
    let output = String::from_utf8(output.stdout).map_err(|e| format!("find: {}", e))?;
    let output = output.split("\n").map(|s| Value::String(s.to_string())).collect();

    Ok(Value::List(output))
}








