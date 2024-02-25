
use caat_rust::Value;
use rand::seq::SliceRandom;




fn find_function<'input>(list: &'input Vec<Value>) -> Result<&'input Value, String> {
    for value in list {
        match value {
            Value::CAATFunction(_) => return Ok(&value),
            _ => continue,
        }
    }
    Err("No function found".to_string())
}

fn find_list<'input>(list: &'input Vec<Value>) -> Result<&'input Value, String> {
    for value in list {
        match value {
            Value::List(_) => return Ok(&value),
            _ => continue,
        }
    }
    Err("No list found".to_string())
}



pub fn map(args: &Vec<Value>) -> Result<Value,String> {
    let list = match find_list(&args)? {
        Value::List(l) => l,
        _ => return Err("No list found".to_string()),
    };
    let function = match find_function(&args)? {
        Value::CAATFunction(f) => f,
        _ => return Err("No function found".to_string()),
    };
    let mut output = Vec::new();
    for value in list.iter() {
        let result = function.call(&vec![value.clone()]);
        output.push(result);
    }
   
    return Ok(Value::List(output.into()));
}


fn fold_find_list(list: &Vec<Value>) -> Result<&Value, String> {
    let mut iter = list.iter();
    let front = iter.next();
    let last = iter.next_back();
    if let Some(Value::List(_)) = front {
        Ok(front.ok_or(String::from("fold: expected list as first argument"))?)
    } else if let Some(Value::List(_)) = last {
        Ok(last.ok_or(String::from("fold: expected list as last argument"))?)
    } else {
        Err("fold: expected list as first or last argument".to_string())
    }
}


fn fold_find_function(list: &Vec<Value>) -> Result<&Value, String> {
    let mut iter = list.iter();
    let front = iter.next();
    let last = iter.next_back();
    if let Some(Value::CAATFunction(_)) = front {
        Ok(front.ok_or(String::from("fold: expected function as first argument"))?)
    } else if let Some(Value::CAATFunction(_)) = last {
        Ok(last.ok_or(String::from("fold: expected function as last argument"))?)
    } else {
        Err("fold: expected list as first or last argument".to_string())
    }
}

pub fn fold(args: &Vec<Value>) -> Result<Value, String> {
    let list = fold_find_list(args)?;
    let function = fold_find_function(args)?;
    let start = match args.get(1) {
        Some(start) => start.clone(),
        None => return Err("fold: expected start value as 2nd argument".to_string()),
    };
    let mut acc = start;
    if let Value::List(list) = list {
        if let Value::CAATFunction(function) = function {
            for value in list.iter() {
                acc = function.call(&[acc, value.clone()]);
            }
        }
    }
    Ok(acc)
}



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

pub fn tail(args: &Vec<Value>) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(list)) => list,
        _ => return Err("tail: expected list as first argument".to_string()),
    };
    let output = list.iter().next_back();
    if let Some(output) = output {
        Ok(output.clone())
    } else {
        Err("tail: empty list".to_string())
    }
}

pub fn rest(args: &Vec<Value>) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(list)) => list,
        _ => return Err("rest: expected list as first argument".to_string()),
    };
    let output = list.iter().skip(1).cloned().collect();
    Ok(Value::List(output))
}


