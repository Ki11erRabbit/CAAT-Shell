use caat_rust::Value;

pub fn contains(args: &Vec<Value>) -> Result<Value,String> {
    if args.len() != 2 {
        return Err("Expected 2 arguments".to_string());
    }
    match (&args[0],&args[1]) {
        (Value::String(s),Value::String(t)) => {
            if s.contains(t) {
                Ok(Value::Boolean(true))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Err("Expected strings".to_string()),
    }
}

pub fn split(args: &Vec<Value>) -> Result<Value,String> {
    let space = Value::String(String::from(" "));
    let split = if args.len() != 2 {
        &space
    } else {
        match &args[1] {
            Value::String(_) => &args[1],
            _ => return Err("Expected string".to_string()),
        }
    };

    match (&args[0],split) {
        (Value::String(s),Value::String(t)) => {
            let v: Vec<Value> = s.split(t).map(|s| Value::String(s.to_string())).collect();
            Ok(Value::List(v.into()))
        }
        _ => Err("Expected strings".to_string()),
    }
}
