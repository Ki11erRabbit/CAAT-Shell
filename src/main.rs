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
        eprintln!("file: {}", file);
        let mut file = parser::parse_file(&file)?;
        eval::run_file(&mut shell, &mut file);
    } else {
        eval::repl(&mut shell);
    }
    Ok(())
}
