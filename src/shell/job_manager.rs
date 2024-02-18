use caat_rust::Value;
use std::collections::HashMap;



pub struct Job {
    command: String,
    id: i64,
    handle: std::thread::JoinHandle<Value>,
}

pub struct JobManager {
    jobs: Vec<Option<Job>>,
    next_id: Option<i64>,
}


impl JobManager {
    pub fn new() -> Self {
        JobManager {
            jobs: vec![],
            next_id: Some(0),
        }
    }
    fn get_next_id(&mut self) {
        for (i , job) in self.jobs.iter().enumerate() {
            if job.is_none() {
                self.next_id = Some(i as i64);
                break;
            }
        }
    }

    pub fn spawn_command(&mut self, command: Value, args: &Vec<Value>) -> Result<Value, String> {
        let id = match self.next_id {
            Some(id) => id,
            None => {
                self.get_next_id();
                match self.next_id {
                    Some(id) => id,
                    None => return Err("background: no id was found".to_string()),
                }
            },
        };
        let cmd = match command {
            Value::CAATFunction(f) => f,
            _ => return Err("background: no command was given".to_string()),
        };
        let command = format!("{}", cmd);
        let args = args.clone();
        let handle = std::thread::spawn(move || {
            cmd.call(&args)
        });
        self.jobs.push(Some(Job {
            command: command.clone(),
            id,
            handle,
        }));
        self.next_id = Some(id + 1);
        let mut output = HashMap::new();
        output.insert(String::from("id"), Value::Integer(id));
        output.insert(String::from("Command"), Value::String(command));
        Ok(Value::Map(output, None))
    }

    pub fn join(&mut self, job: Value) -> Result<Value, String> {
        let id = match job {
            Value::Map(members, _) => {
                let mut id = None;
                for (key, value) in members {
                    if key == "id" {
                        match value {
                            Value::Integer(i) => id = Some(i),
                            _ => return Err("join: id is not an integer".to_string()),
                        }
                    } else {
                        continue;
                    }
                }
                match id {
                    Some(i) => i,
                    None => return Err("join: no id was given".to_string()),
                }
            },
            Value::Integer(i) => i,
            _ => return Err("join: no job map or job id was given".to_string()),
        };
        let job = match self.jobs.get_mut(id as usize) {
            Some(job) => job,
            None => return Err("join: no job with that id".to_string()),
        };
        if let Some(job) = job.take() {
            self.next_id = None;
            Ok(job.handle.join().map_err(|_| "join: thread panicked".to_string())?)
        } else {
            return Err("join: no job with that id".to_string());
        }
    }
    
    pub fn jobs(&self, args: &Vec<Value>) -> Value {
        let mut output = Vec::new();
        for job in self.jobs.iter() {
            if let Some(job) = job {
                output.push(Value::List(vec![Value::Integer(job.id), Value::String(job.command.clone())].into()));
            }
        }
        return Value::List(output.into());
    }
}


