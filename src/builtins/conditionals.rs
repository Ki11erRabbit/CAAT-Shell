use caat_rust::Value;


pub fn if_command(args: &Vec<Value>) -> Result<Value,String> {
    let test = match args.get(0) {
        Some(Value::CAATFunction(f)) => f,
        _ => return Err("No test provided".to_string()),
    };
    let if_true = match args.get(1) {
        Some(Value::CAATFunction(f)) => f,
        _ => return Err("No if true provided".to_string()),
    };
    let if_false = match args.get(2) {
        Some(Value::CAATFunction(f)) => f,
        _ => return Err("No if false provided".to_string()),
    };
    match test.call(&vec![]) { 
        Value::Boolean(true) => {
            return Ok(if_true.call(&vec![]));
        }
        Value::Boolean(false) => {
            return Ok(if_false.call(&vec![]));
        }
        _ => return Err("Type Error: Test did not return a boolean".to_string()),
    }
}
