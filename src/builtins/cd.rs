
use caat_rust::Value;






pub fn cd(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::String(env!("HOME").to_string())
    } else {
        return args[0].clone()}
}
