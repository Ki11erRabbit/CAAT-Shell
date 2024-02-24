
use caat_rust::Value;
use rand::seq::SliceRandom;








pub fn shuf(args: &Vec<Value>) -> Result<Value,String> {
    let mut rng = rand::thread_rng();
    let list = match args.get(0) {
        Some(Value::List(list)) => list.clone(),
        _ => return Err("shuf: expected list as first argument".to_string()),
    };
    let mut output = list;
    output.shuffle(&mut rng);
    Ok(Value::List(output.clone()))
}

pub fn head(args: &Vec<Value>) -> Result<Value,String> {
    let list = match args.get(0) {
        Some(Value::List(list)) => list,
        _ => return Err("head: expected list as first argument".to_string()),
    };
    let output = list.iter().next();
    if let Some(output) = output {
        Ok(output.clone())
    } else {
        Err("head: empty list".to_string())
    }
}



