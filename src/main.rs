use parser::{parse_shebang, File};
use shell::Shell;
use std::sync::{Arc, RwLock};

pub mod parser;
pub mod eval;
#[macro_use]
pub mod shell;
pub mod builtins;

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let shell = Arc::new(RwLock::new(Shell::new()));
    let args: Vec<String> = std::env::args().collect();
    //eprintln!("args: {:?}", args);
    //eprintln!("args.len(): {}", args.len());
    if args.len() > 1 {
        let mut file = parse_file(&args[1])?;
        eval::run_file(shell, &mut file);
    } else {
        eval::repl(shell);
    }
    Ok(())
}

fn parse_file(file_path: &str) -> Result<File, Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(file_path)?;
    match parse_shebang(&file) {
        Ok(shebang) => {
            //eprintln!("shebang: {}", shebang);
            if !shebang.contains("caat_shell") {
                let mut command = std::process::Command::new(shebang);
                command.arg(file_path);
                let status = command.status()?;
                if !status.success() {
                    eprintln!("error: failed to execute shebang");
                    std::process::exit(1);
                }
                //eprintln!("status: {:?}", status);
                std::process::exit(0);
            } else {
                let file = parser::parse_file(&file)?;
                return Ok(file);
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
        }
    }

    //eprintln!("file: {}", file);
    let file = parser::parse_file(&file)?;
    Ok(file)
}

