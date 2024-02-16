use caat_rust::Value;




pub fn echo(args: Vec<Value>) -> Value {
    let mut result = String::new();
    for arg in args {
        result.push_str(&arg.to_string());
        result.push_str(" ");
    }
    let result = result.trim().to_string();
    Value::String(result)
}
