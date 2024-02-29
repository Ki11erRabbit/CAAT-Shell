use std::collections::HashMap;
use caat_rust::Value;
use job_manager::JobManager;
pub mod job_manager;
pub mod function;

#[macro_export]
macro_rules! borrow_mut {
    ($e:expr) => {
        loop {
            match $e.try_write() {
                Ok(e) => break e,
                Err(std::sync::TryLockError::WouldBlock) => continue,
                Err(e) => panic!("error: {:?}", e),
            }
        }
    }
}

#[macro_export]
macro_rules! borrow {
    ($e:expr) => {
        loop {
            match $e.try_read() {
                Ok(e) => break e,
                Err(std::sync::TryLockError::WouldBlock) => continue,
                Err(e) => panic!("error: {:?}", e),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shell {
    environment: Environment,
    job_manager: JobManager,
    functions: HashMap<String, function::Function>,
}


impl Shell {
    pub fn new() -> Self {
        Shell {
            environment: Environment::new(),
            job_manager: JobManager::new(),
            functions: HashMap::new(),
        }
    }
    pub fn with_environment(environment: Environment) -> Self {
        Shell {
            environment,
            job_manager: JobManager::new(),
            functions: HashMap::new(),
        }
    }
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }
    pub fn job_manager(&self) -> &JobManager {
        &self.job_manager
    }
    pub fn job_manager_mut(&mut self) -> &mut JobManager {
        &mut self.job_manager
    }
    pub fn get_function(&self, name: &str) -> Option<function::Function> {
        match self.functions.get(name) {
            Some(function) => {
                let function = function.clone();
                Some(function)
            }
            None => None,
        }
    }
    pub fn set_function(&mut self, name: String, function: function::Function) {
        self.functions.insert(name, function);
    }
    pub fn merge(&mut self, other: Shell) {
        self.environment.global.extend(other.environment.global);
        for scope in other.environment.scoped {
            self.environment.scoped.push(scope);
        }
        self.functions.extend(other.functions);
    }
}


#[derive(Debug, Clone)]
pub struct Environment {
    global: HashMap<String, Value>,
    scoped: Vec<HashMap<String, Value>>,
}


impl Environment {
    pub fn create_global() -> HashMap<String, Value> {
        let mut global = HashMap::new();
        global.insert("HOME".to_string(), Value::String(env!("HOME").to_string()));
        global
    }
    pub fn new() -> Self {
        let global = Environment::create_global();
        Environment {
            global,
            scoped: vec![HashMap::new()],
        }
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scoped.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        self.global.get(name)
    }
    pub fn set(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scoped.last_mut() {
            scope.insert(name, value);
        } else {
            self.global.insert(name, value);
        }
    }
    pub fn remove(&mut self, name: &str) {
        for scope in self.scoped.iter_mut().rev() {
            if scope.remove(name).is_some() {
                return;
            }
        }
        self.global.remove(name);
    }
    pub fn push_scope(&mut self) {
        self.scoped.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scoped.pop();
    }
    pub fn get_current(&self) -> HashMap<String, Value> {
        if let Some(scope) = self.scoped.last() {
            scope.clone()
        } else {
            self.global.clone()
        }
    }
    pub fn extend_current(&mut self, other: &HashMap<String, Value>) {
        if let Some(scope) = self.scoped.last_mut() {
            scope.extend(other.clone());
        } else {
            self.global.extend(other.clone());
        }
    }
}


