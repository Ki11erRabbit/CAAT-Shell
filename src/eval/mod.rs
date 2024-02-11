use crate::parser::Statement;
use std::io::Write;






pub fn repl() {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let mut interactive = crate::parser::parse_interactive(&input).unwrap();
        eval(&mut interactive);
    }
}




fn eval(input: &mut dyn Iterator<Item = Statement>) {
    match input.next() {
        Some(Statement::Assignment(assignment)) => {
            println!("Assignment: {:?} = {:?}", assignment.target, assignment.value);
        }
        Some(Statement::Expression(expression)) => {
            println!("Expression: {:?}", expression);
        }
        None => {}
    }
}
