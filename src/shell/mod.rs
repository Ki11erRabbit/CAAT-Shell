use std::collections::HashMap;
use caat_rust::Value;
use job_manager::JobManager;

pub mod job_manager;

pub struct Shell {
    environment: Environment,
    job_manager: JobManager,
}


impl Shell {
    pub fn new() -> Self {
        Shell {
            environment: Environment::new(),
            job_manager: JobManager::new(),
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
}



pub struct Environment {
    global: HashMap<String, Value>,
    scoped: Vec<HashMap<String, Value>>,
}


impl Environment {
    pub fn new() -> Self {
        let global = HashMap::new();
        Environment {
            global,
            scoped: vec![],
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
    pub fn push_scope(&mut self) {
        self.scoped.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scoped.pop();
    }
}


