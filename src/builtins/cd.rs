
use caat_rust::Value;
use std::env;





pub fn cd(args: &Vec<Value>) -> Result<Value, String> {
    if args.len() == 0 {
        match env::var("HOME") {
            Ok(home) => {
                let _ = env::set_current_dir(home).map_err(|e| e.to_string())?;
            }
            Err(_) => {
                println!("HOME not set");
            }
        }
    } else {
        match args[0] {
            Value::String(ref path) => {
                let _ = env::set_current_dir(path).map_err(|e| e.to_string())?;
            }
            _ => {
                println!("cd: expected string");
            }
        }
    }
    return Ok(Value::Null)
}
