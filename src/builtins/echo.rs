use caat_rust::Value;




pub fn echo(args: Vec<Value>) -> Value {
    return args[0].clone()
}
