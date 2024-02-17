
use caat_rust::Value;
use std::env;





pub fn cd(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        match env::var("HOME") {
            Ok(home) => {
                let _ = env::set_current_dir(home);
            }
            Err(_) => {
                println!("HOME not set");
            }
        }
    } else {
        match args[0] {
            Value::String(ref path) => {
                let _ = env::set_current_dir(path);
            }
            _ => {
                println!("cd: expected string");
            }
        }
    }
    return Value::Null
}
