use caat_rust::{Caat, Value};
use std::cell::RefCell;
use crate::parser::File;

use super::Shell;





#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: File,
    pub shell: RefCell<Option<Shell>>,
}

impl Function {
    pub fn new(name: &str, arguments: Vec<String>, body: File) -> Self {
        Function {
            name: name.to_string(),
            arguments,
            body: body,
            shell: RefCell::new(None),
        }
    }
    
    pub fn attach_shell(&mut self, shell: Shell) {
        let mut current = self.shell.borrow_mut();
        match *current {
            Some(ref mut current) => {
                current.merge(shell);
            }
            None => {
                *current = Some(shell);

            }
        }
    }
}

unsafe impl Sync for Function {}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Function: {}", self.name)
    }
}

impl Caat for Function {
    fn call(&self, args: &[Value]) -> Value {
        if let Some(mut shell) = self.shell.take() {
            let environment = shell.environment_mut();
            for (i, arg) in self.arguments.iter().enumerate() {
                if let Some(value) = args.get(i) {
                    environment.set(arg.clone(), value.clone());
                }
            }
            let value = crate::eval::run_file(&mut shell, &mut self.body.clone());
             
            *self.shell.borrow_mut() = Some(shell);
            value
        } else {
            return Value::Failure("Shell not found".to_string());
        }
    }
}
