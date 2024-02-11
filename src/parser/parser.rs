use chumsky::prelude::*;

use super::{Command, Expression, Literal, Operator, Pipeline};









fn integer_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
    let sign = choice((
        just('+'),
        just('-'),
                     ));

    choice((
            sign.then(text::int(10))
                .map(|(sign, value)| if sign == '-' { -value.parse::<i64>().expect("value was a bad number") } else { value.parse::<i64>().expect("value was a bad number") }),
            text::int(10)
                .map(|value: String| value.parse::<i64>().expect("value was a bad number")),
            ))
        .map(Literal::Integer)
}



fn float_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
    let sign = choice((
        just('+'),
        just('-'),
                     ));
    let decimal = choice((
            text::int(10).map(|value| value),
            text::digits(10).then(text::int(10)).map(|(left, right): (String, String)| left + right.as_str()),
                         ));
    let float = text::digits(10).then_ignore(just('.')).then(decimal).map(|(left, right)| format!("{}.{}", left, right));
    choice((
            sign.then(float)
                .map(|(sign, value)| if sign == '-' { -value.parse::<f64>().expect("value was a bad number") } else { value.parse::<f64>().expect("value was a bad number") }),
            float
                .map(|value: String| value.parse::<f64>().expect("value was a bad number")),
            ))
        .map(Literal::Float)
}


fn string_parser() -> impl Parser<char, Literal, Error = Simple<char>> {

    let escape = just::<char, char, Simple<char>>('\\')
        .then(one_of("\"\\nrt "))
        .map(|(_, c)| match c {
            '"' => '"',
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            ' ' => ' ',
            _ => unreachable!()
        });

    let string_char = none_of("\"\\")
        .or(escape);

    let string = just('"')
        .ignore_then(string_char.repeated())
        .then_ignore(just('"'))
        .then_ignore(end())
        .map(|s| s.iter().collect());

    string.map(Literal::String)
}

fn string_parser_unescaped() -> impl Parser<char, Literal, Error = Simple<char>> {
    string_parser_unescaped_as_str().map(Literal::String)
}

fn string_parser_unescaped_as_str() -> impl Parser<char, String, Error = Simple<char>> {
    let string = none_of("\"\\: ")
        .repeated()
        .map(|s| s.iter().collect());
    string
}


fn boolean_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
    choice((
            text::keyword("true").map(|_| true),
            text::keyword("false").map(|_| false),
            ))
        .map(Literal::Boolean)
}


fn literal_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
    recursive(|lst_map| {
        let member = string_parser_unescaped_as_str().then_ignore(just(':').padded()).then(lst_map.clone()).map(|(key, value)| (key, value));
        choice((
                float_parser(),
                integer_parser(),
                string_parser(),
                boolean_parser(),
                lst_map.clone().separated_by(just(',').padded())
                    .delimited_by(just('[').padded(),just(']').padded())
                    .map(Literal::List),
                member
                    .separated_by(just(',').padded())
                    .delimited_by(just('{').padded(),just('}').padded())
                    .collect()
                    .map(|arg| Literal::Map(arg)),
                ))
    })
}


/*fn command_parser() -> impl Parser<char, Command, Error = Simple<char>> {
    let name = string_parser_unescaped_as_str();
    let arguments = literal_parser().repeated().padded();
    name.then(arguments).map(|(name, arguments)| Command { name, arguments })
}

fn pipeline_parser() -> impl Parser<char, Pipeline, Error = Simple<char>> {
    let operator = choice((
        just("|").map(|_| super::Operator::Pipe),
        just("&&").map(|_| super::Operator::And),
        just("||").map(|_| super::Operator::Or),
        just(";").map(|_| super::Operator::Then),
    ));
    recursive(|pipeline| {
        choice((
                command_parser().then(operator.padded()).then(pipeline).map(|((command, operator), next)| Pipeline { command, operator: Some(operator), next: Some(Box::new(next)) }),
                command_parser().map(|command| Pipeline { command, operator: None, next: None }),
                ))
    })
}*/

fn expression_parser() -> impl Parser<char, Expression, Error = Simple<char>> {
    let operator = choice((
        just("|").map(|_| super::Operator::Pipe),
        just("&&").map(|_| super::Operator::And),
        just("||").map(|_| super::Operator::Or),
        just(";").map(|_| super::Operator::Then),
    ));

    recursive(|expression| {
        let parenthesized = just('(').padded().ignore_then(expression.clone()).then_ignore(just(')').padded()).map(|arg| Expression::Parenthesized(Box::new(arg)));
        let variable = just('$').ignore_then(string_parser_unescaped_as_str()).map(|arg| Expression::Variable(arg));
        let literal = literal_parser().map(Expression::Literal);
        let command = string_parser_unescaped_as_str().padded().then(choice((
                    parenthesized.clone(),
                    just('$').ignore_then(string_parser_unescaped_as_str()).map(|arg| Expression::Variable(arg)),
                    literal_parser().map(Expression::Literal),
                    )).repeated()).map(|(name, arguments)| Command { name, arguments });
        let pipeline = recursive(|pipeline| {
            choice((
                string_parser_unescaped_as_str().padded().then(choice((
                    literal_parser().map(Expression::Literal),
                    just('$').ignore_then(string_parser_unescaped_as_str()).map(|arg| Expression::Variable(arg)),
                    parenthesized.clone(),
                    )).repeated()).map(|(name, arguments)| Command { name, arguments }).then(operator.padded()).then(pipeline.padded()).map(|((command, operator), next)| Pipeline { command, operator: Some(operator), next: Some(Box::new(next)) }),
                string_parser_unescaped_as_str().padded().then(choice((
                    literal_parser().map(Expression::Literal),
                    just('$').ignore_then(string_parser_unescaped_as_str()).map(|arg| Expression::Variable(arg)),
                    parenthesized.clone(),
                    )).repeated()).map(|(name, arguments)| Command { name, arguments }).map(|command| Pipeline { command, operator: None, next: None }),
                              ))
        }).map(Expression::Pipeline);
        choice((
            literal,
            parenthesized,
            pipeline,
            variable,
            ))
    })
}

fn assignment_parser() -> impl Parser<char, super::Assignment, Error = Simple<char>> {
    let target = string_parser_unescaped_as_str();
    let value = expression_parser();
    target.then_ignore(just('=').padded()).then(value).map(|(target, value)| super::Assignment { target, value })
}

fn statement_parser() -> impl Parser<char, super::Statement, Error = Simple<char>> {
    choice((
            assignment_parser().map(super::Statement::Assignment),
            expression_parser().map(super::Statement::Expression),
            ))
}


fn file_parser() -> impl Parser<char, super::File, Error = Simple<char>> {
    statement_parser().repeated().map(|statements| super::File { statements })
}

fn interactive_parser() -> impl Parser<char, super::Interactive, Error = Simple<char>> {
    statement_parser().map(|statement| super::Interactive { statement, gave_statement: false})
}


pub fn parse_file(input: &str) -> Result<super::File, ()> {
    file_parser().parse(input).map(|value| value).map_err(|_| ())
}

pub fn parse_interactive(input: &str) -> Result<super::Interactive, ()> {
    interactive_parser().parse(input).map(|value| value).map_err(|_| ())
}




#[cfg(test)]
mod tests {
    use super::*;

    pub fn parse(input: &str) -> Result<Literal, ()> {
        literal_parser().parse(input).map(|value| value).map_err(|_| ())
    }
    
    #[test]
    fn test_parse_integer() {
        assert_eq!(parse("123"), Ok(Literal::Integer(123)));
        assert_eq!(parse("-123"), Ok(Literal::Integer(-123)));
    }
    
    #[test]
    fn test_parse_float() {
        assert_eq!(parse("123.456"), Ok(Literal::Float(123.456)));
        assert_eq!(parse("-123.456"), Ok(Literal::Float(-123.456)));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse("\"hello, world\""), Ok(Literal::String("hello, world".to_string())));
        assert_eq!(parse("\"hello, \\\"world\\\"\""), Ok(Literal::String("hello, \"world\"".to_string())));
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse("true"), Ok(Literal::Boolean(true)));
        assert_eq!(parse("false"), Ok(Literal::Boolean(false)));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse("[]"), Ok(Literal::List(vec![])));
        assert_eq!(parse("[1, 2, 3]"), Ok(Literal::List(vec![Literal::Integer(1), Literal::Integer(2), Literal::Integer(3)])));
    }

    #[test]
    fn test_parse_map() {
        assert_eq!(parse("{}"), Ok(Literal::Map(vec![])));
        assert_eq!(parse("{a: 1, b: 2}"), Ok(Literal::Map(vec![("a".to_string(), Literal::Integer(1)), ("b".to_string(), Literal::Integer(2))])));
    }
}


