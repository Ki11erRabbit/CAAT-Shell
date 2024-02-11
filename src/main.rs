pub mod parser;
pub mod eval;

fn main() {
    crate::eval::repl();
}
