use caat_rust::{Caat, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::parser::File;
use crate::borrow_mut;

use super::Shell;





#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: File,
    pub shell: Arc<RwLock<Shell>>,
    pub environment: Option<HashMap<String, Value>>,
}

impl Function {
    pub fn new(name: &str, arguments: Vec<String>, body: File, shell: Arc<RwLock<Shell>>) -> Self {
        Function {
            name: name.to_string(),
            arguments,
            body,
            shell,
            environment: None,
        }
    }
    pub fn bind_environment(&mut self, environment: HashMap<String, Value>) {
        self.environment = Some(environment);
    }
}




impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Function: {}", self.name)
    }
}

impl Caat for Function {
    fn call(&self, args: &[Value]) -> Value {
        let mut borrowed_shell = borrow_mut!(self.shell);
        let environment = borrowed_shell.environment_mut();
        environment.push_scope();
        match &self.environment {
            Some(env) => {
                environment.extend_current(env);
            }
            None => {}
        }
        for (i, arg) in self.arguments.iter().enumerate() {
            if let Some(value) = args.get(i) {
                environment.set(arg.clone(), value.clone());
            }
        }
        drop(borrowed_shell);
        let value = crate::eval::run_file(self.shell.clone(), &mut self.body.clone());
         
        let mut borrowed_shell = borrow_mut!(self.shell);
        let environment = borrowed_shell.environment_mut();
        environment.pop_scope();
        value.get_value()
    }
}
