use std::fs;
use std::fs::Metadata;
use caat_rust::Value;
use std::os::unix::fs::PermissionsExt;
use chrono::prelude::*;
use chrono::Utc;
use std::collections::HashMap;

struct Ls {
    show_all: bool,
    show_long: bool,
    path: Option<String>,
}

impl Ls {
    fn new() -> Ls {
        Ls {
            show_all: false,
            show_long: false,
            path: None,
        }
    }
    
    fn parse_arguments(&mut self, args: &Vec<Value>) {
        for arg in args {
            match arg {
                Value::String(s) => {
                    match s.as_str() {
                        "-a" => self.show_all = true,
                        "-l" => self.show_long = true,
                        _ => self.path = Some(s.to_string()),
                    }
                },
                Value::Map(map, _) => {
                    if let Some(s) = map.get("type") {
                        if let Value::String(s) = s {
                            if s == "dir_entry" {
                                if let Some(s) = map.get("full_path") {
                                    if let Value::String(s) = s {
                                        self.path = Some(s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn output(&self) -> Result<Value, String> {
        let paths = if let Some(path) = &self.path {
            fs::read_dir(path).map_err(|e| e.to_string())?
        } else {
            fs::read_dir(".").map_err(|e| e.to_string())?
        };
        let mut result = Vec::new();
        for path in paths {
            let path = path.map_err(|e| e.to_string())?.path();
            let metadata = path.metadata().map_err(|e| e.to_string())?;
            let modified_time = metadata.modified().map_err(|e| e.to_string())?;
            let modified_time = DateTime::<Utc>::from(modified_time);
            let modified_time = modified_time.format("%Y-%m-%d %H:%M").to_string();
            let size = metadata.len();
            let metadata = Self::metadata_string(metadata);
            let name = path.file_name().ok_or("unable to get file_name".to_string())?.to_str().ok_or("unable to get str".to_string())?;
            if name.starts_with(".") && !self.show_all {
                continue;
            }
            let metadata = Value::String(metadata);
            let size = Value::Integer(size as i64);
            let modified_time = Value::String(modified_time);
            let name = Value::String(name.to_string());


            let mut obj_hash = HashMap::new();
            obj_hash.insert("type".to_string(), Value::String("dir_entry".to_string()));
            obj_hash.insert("metadata".to_string(), metadata);
            obj_hash.insert("size".to_string(), size);
            obj_hash.insert("modified_time".to_string(), modified_time);
            obj_hash.insert("name".to_string(), name);
            obj_hash.insert("full_path".to_string(), Value::String(path.to_str().ok_or(String::from("bad path"))?.to_string()));

            let format = if self.show_long {
                Some(String::from("{metadata} {size} {modified_time} {name}"))
            } else {
                Some(String::from("{name}"))
            };
            
            
            result.push(Value::Map(obj_hash, format));
        }
        return Ok(result.into())
    }
    
    

    fn metadata_string(metadata: Metadata) -> String {
        let mut output = String::new();
        if metadata.is_dir() {
            output.push('d');
        } else if metadata.is_symlink() {
            output.push('l');
        } else {
            output.push('-');
        }
        let permissions_mode = metadata.permissions().mode();
        if permissions_mode & 0o400 != 0 {
            output.push('r');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o200 != 0 {
            output.push('w');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o100 != 0 {
            output.push('x');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o40 != 0 {
            output.push('r');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o20 != 0 {
            output.push('w');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o10 != 0 {
            output.push('x');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o4 != 0 {
            output.push('r');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o2 != 0 {
            output.push('w');
        } else {
            output.push('-');
        }
        if permissions_mode & 0o1 != 0 {
            output.push('x');
        } else {
            output.push('-');
        }
        output
    }
}



pub fn ls(args: &Vec<Value>) -> Result<Value,String> {
    let mut ls = Ls::new();
    ls.parse_arguments(args);
    return ls.output();
}
