use caat_rust::Value;



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
        eprintln!("result: {:#?}", result);
        output.push(result);
    }
   
    return Ok(Value::List(output.into()));
}
