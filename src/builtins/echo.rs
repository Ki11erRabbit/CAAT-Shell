use caat_rust::Value;




pub fn echo(args: &Vec<Value>) -> Value {
    return args[0].clone()
}


pub fn trace(args: &Vec<Value>) -> Value {
    println!("{:?}",args[0]);
    return args[0].clone()
}

