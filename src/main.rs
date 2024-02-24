use parser::parse_shebang;
use shell::Shell;

pub mod parser;
pub mod eval;
pub mod shell;
pub mod builtins;

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let mut shell = Shell::new();
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let file = std::fs::read_to_string(&args[1])?;
        match parse_shebang(&file) {
            Ok(shebang) => {
                let mut command = std::process::Command::new(shebang);
                command.arg(&args[1]);
                let status = command.status()?;
                if !status.success() {
                    eprintln!("error: failed to execute shebang");
                    std::process::exit(1);
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }

        //eprintln!("file: {}", file);
        let mut file = parser::parse_file(&file)?;
        //eprintln!("file: {:#?}", file);
        eval::run_file(&mut shell, &mut file);
    } else {
        eval::repl(&mut shell);
    }
    Ok(())
}
