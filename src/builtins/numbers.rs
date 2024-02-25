use either::Either;
use caat_rust::Value;






pub fn add(args: &Vec<Value>) -> Result<Value,String> {
    let mut sum: Either<i64,f64> = Either::Left(0);
    for arg in args.iter() {
        match arg {
            Value::Integer(i) => {
                match sum {
                    Either::Left(ref mut s) => *s += i,
                    Either::Right(_) => return Err("Cannot add integer to float".to_string()),
                }
            }
            Value::Float(f) => {
                match sum {
                    Either::Left(0) => {
                        sum = Either::Right(*f);
                    }
                    Either::Left(_) => return Err("Cannot add float to integer".to_string()),
                    Either::Right(ref mut s) => *s += f,
                }
            }
            _ => return Err("Expected number".to_string()),
        }
    }
    match sum {
        Either::Left(s) => Ok(Value::Integer(s)),
        Either::Right(s) => Ok(Value::Float(s)),
    }
}


pub fn sub(args: &Vec<Value>) -> Result<Value,String> {
    let mut sum: Either<i64,f64> = Either::Left(0);
    for arg in args.iter() {
        match arg {
            Value::Integer(i) => {
                match sum {
                    Either::Left(ref mut s) => *s -= i,
                    Either::Right(_) => return Err("Cannot add integer to float".to_string()),
                }
            }
            Value::Float(f) => {
                match sum {
                    Either::Left(0) => {
                        sum = Either::Right(*f);
                    }
                    Either::Left(_) => return Err("Cannot add float to integer".to_string()),
                    Either::Right(ref mut s) => *s -= f,
                }
            }
            _ => return Err("Expected number".to_string()),
        }
    }
    match sum {
        Either::Left(s) => Ok(Value::Integer(s)),
        Either::Right(s) => Ok(Value::Float(s)),
    }
}



pub fn mult(args: &Vec<Value>) -> Result<Value,String> {
    let mut sum: Either<i64,f64> = Either::Left(1);
    for arg in args.iter() {
        match arg {
            Value::Integer(i) => {
                match sum {
                    Either::Left(ref mut s) => *s *= i,
                    Either::Right(_) => return Err("Cannot add integer to float".to_string()),
                }
            }
            Value::Float(f) => {
                match sum {
                    Either::Left(1) => {
                        sum = Either::Right(*f);
                    }
                    Either::Left(_) => return Err("Cannot add float to integer".to_string()),
                    Either::Right(ref mut s) => *s *= f,
                }
            }
            _ => return Err("Expected number".to_string()),
        }
    }
    match sum {
        Either::Left(s) => Ok(Value::Integer(s)),
        Either::Right(s) => Ok(Value::Float(s)),
    }
}


pub fn div(args: &Vec<Value>) -> Result<Value,String> {
    let mut args = args.iter();
    let mut sum: Either<i64,f64> = match args.next() {
        Some(Value::Integer(i)) => Either::Left(*i),
        Some(Value::Float(f)) => Either::Right(*f),
        _ => return Err("Expected number".to_string()),
    };
    for arg in args {
        match arg {
            Value::Integer(0) => {
                return Err("Division by zero".to_string());
            },
            Value::Integer(i) => {
                match sum {
                    Either::Left(ref mut s) => *s /= i,
                    Either::Right(_) => return Err("Cannot add integer to float".to_string()),
                }
            }
            Value::Float(x) if *x == 0.0 => {
                return Err("Division by zero".to_string());
            },
            Value::Float(f) => {
                match sum {
                    Either::Left(_) => return Err("Cannot add float to integer".to_string()),
                    Either::Right(ref mut s) => *s /= f,
                }
            }
            _ => return Err("Expected number".to_string()),
        }
    }
    match sum {
        Either::Left(s) => Ok(Value::Integer(s)),
        Either::Right(s) => Ok(Value::Float(s)),
    }
}
