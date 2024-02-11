use shell::Shell;

pub mod parser;
pub mod eval;
pub mod shell;

fn main() {
    let mut shell = Shell::new();
    crate::eval::repl(&mut shell);
}
