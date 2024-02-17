use std::fs;
use caat_rust::Value;


struct Ls {
    show_all: bool,
    show_long: bool,
}

impl Ls {
    fn new() -> Ls {
        Ls {
            show_all: false,
            show_long: false,
        }
    }
    
    fn parse_arguments(&mut self, args: Vec<Value>) {
        for arg in args {
            match arg {
                Value::String(s) => {
                    match s.as_str() {
                        "-a" => self.show_all = true,
                        "-l" => self.show_long = true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    
    fn output(&self) -> Value {
        if self.show_long {
            return self.long_output();
        } else {
            return self.short_output();
        }
    }
    
    fn short_output(&self) -> Value {
        let paths = fs::read_dir(".").unwrap();
        let mut result = Vec::new();
        for path in paths {
            let path = path.unwrap().path();
            match path.file_name() {
                None => continue,
                Some(name) => {
                    let name = name.to_str().unwrap();
                    if name.starts_with(".") && !self.show_all {
                        continue;
                    }
                    result.push(Value::String(name.to_string()));
                }
            }
        }
        return Value::List(result.into())
    }
    
    fn long_output(&self) -> Value {
        let paths = fs::read_dir(".").unwrap();
        let mut result = Vec::new();
        for path in paths {
            let path = path.unwrap().path();
            let metadata = path.metadata().unwrap();
            let name = path.file_name().unwrap().to_str().unwrap();
            let size = metadata.len();
            let size = Value::Integer(size as i64);
            let name = Value::String(name.to_string());
            result.push(Value::List(vec![size, name].into()));
        }
        return result.into()
    }
}



pub fn ls(args: Vec<Value>) -> Value {
    let mut ls = Ls::new();
    ls.parse_arguments(args);
    return ls.output();
}
