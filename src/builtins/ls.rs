use std::fs;
use std::fs::Metadata;
use caat_rust::Value;
use std::os::unix::fs::PermissionsExt;
use chrono::prelude::*;
use chrono::Utc;

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
    
    fn parse_arguments(&mut self, args: Vec<Value>) {
        for arg in args {
            match arg {
                Value::String(s) => {
                    match s.as_str() {
                        "-a" => self.show_all = true,
                        "-l" => self.show_long = true,
                        _ => self.path = Some(s),
                    }
                }
                _ => {}
            }
        }
    }
    
    fn output(&self) -> Result<Value, String> {
        if self.show_long {
            return self.long_output();
        } else {
            return self.short_output();
        }
    }
    
    fn short_output(&self) -> Result<Value, String> {
        let paths = fs::read_dir(".").map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for path in paths {
            let path = path.map_err(|e| e.to_string())?.path();
            match path.file_name() {
                None => continue,
                Some(name) => {
                    let name = name.to_str().ok_or("Failed to turn path into str".to_string())?;
                    if name.starts_with(".") && !self.show_all {
                        continue;
                    }
                    result.push(Value::String(name.to_string()));
                }
            }
        }
        return Ok(Value::List(result.into()))
    }
    
    fn long_output(&self) -> Result<Value, String> {
        let paths = fs::read_dir(".").map_err(|e| e.to_string())?;
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
            let metadata = Value::String(metadata);
            let size = Value::Integer(size as i64);
            let modified_time = Value::String(modified_time);
            let name = Value::String(name.to_string());
            result.push(Value::List(vec![metadata, size, modified_time, name].into()));
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



pub fn ls(args: Vec<Value>) -> Result<Value,String> {
    let mut ls = Ls::new();
    ls.parse_arguments(args);
    return ls.output();
}
