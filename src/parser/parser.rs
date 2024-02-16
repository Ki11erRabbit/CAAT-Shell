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
        .map(|f| Literal::Float(f.to_string()))
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
    let string = none_of("\"\\:")
        .repeated()
        .map(|s| s.iter().collect());
    string
}

fn identifier_parser() -> impl Parser<char, String, Error = Simple<char>> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Special(char),
    Identifier(String),
    Literal(Literal),
    Operator(Operator),
}

fn tokenizer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    choice((
            literal_parser().map(|x| {eprintln!("{:?}", x); Token::Literal(x)}).padded(),
            identifier_parser().map(|x| {eprintln!("{}", x); Token::Identifier(x)}).padded(),
            just('|').map(|_| Token::Operator(Operator::Pipe)).padded(),
            just("&&").map(|_| Token::Operator(Operator::And)).padded(),
            just("||").map(|_| Token::Operator(Operator::Or)).padded(),
            just(';').map(|_| Token::Operator(Operator::Then)).padded(),
            just('$').map(|_| Token::Special('$')).padded(),
            just('(').map(|_| Token::Special('(')).padded(),
            just(')').map(|_| Token::Special(')')).padded(),
            )).padded().repeated().labelled("tokens")
}


fn variable_parser() -> impl Parser<Token, Expression, Error = Simple<Token>> {
    just(Token::Special('$')).ignore_then(filter_map(|span, token| {
        match token {
            Token::Identifier(name) => Ok(Expression::Variable(name)),
            _ => Err(Simple::custom(span, String::from("identifier"))),
        }
    }))
}

fn expression_parser() -> impl Parser<Token, Expression, Error = Simple<Token>> {
    /*let operator = choice((
        just("|").map(|_| super::Operator::Pipe),
        just("&&").map(|_| super::Operator::And),
        just("||").map(|_| super::Operator::Or),
        just(";").map(|_| super::Operator::Then),
    ));*/

    recursive(|expression| {
        let parenthesized = just(Token::Special('(')).ignore_then(expression.clone()).then_ignore(just(Token::Special(')'))).map(|arg| Expression::Parenthesized(Box::new(arg)));
        let literal = filter_map(|span, token| {
            match token {
            Token::Literal(literal) => Ok(Expression::Literal(literal)),
            _ => Err(Simple::custom(span, String::from("literal"))),
            }
        });
        let argument = choice((
                    //parenthesized.clone(),
                    variable_parser(),//just('$').ignore_then(string_parser_unescaped_as_str()).map(|arg| Expression::Variable(arg)),
                    literal,
                    ));
        let argument_list = argument.repeated().labelled("argument list");

        let command = filter_map(|span, token| {
            eprintln!("{:?}", token);
            match token {
            Token::Identifier(name) => Ok(name),
            _ => Err(Simple::custom(span, String::from("identifier"))),
            }
        }).repeated().at_least(1).map(|list| {
            eprintln!("{:?}", list);
            let mut iter = list.into_iter();
            let name = iter.next().unwrap();
            let arguments: Vec<String> = iter.collect();
            format!("{} {}", name, arguments.join(" "))
        }).then(argument_list).map(|(name, arguments)| { eprintln!("{:?}", arguments); Command { name, arguments }});

        //let command = command_name_parser().padded().then(argument_list).map(|(name, arguments)| { eprintln!("{:?}", arguments); Command { name, arguments }});
        /*let pipeline = recursive(|pipeline| {
            choice((
                command_name_parser()
                    .padded()
                    .then(argument.repeated())
                    .map(|(name, arguments)| Command { name, arguments })
                    .then(operator.padded())
                    .then(pipeline.padded())
                    .map(|((command, operator), next)| Pipeline { command, operator: Some(operator), next: Some(Box::new(next)) }),
                              ))
        }).map(Expression::Pipeline);*/
        choice((
            literal,
            parenthesized,
            //pipeline,
            variable_parser(),
            command.map(|cmd| Expression::Pipeline(Pipeline { command: cmd, operator: None, next: None })),
            ))
    })
}

fn assignment_parser() -> impl Parser<Token, super::Assignment, Error = Simple<Token>> {
    let target = filter_map(|span, token| {
        match token {
            Token::Identifier(name) => Ok(name),
            _ => Err(Simple::custom(span, String::from("identifier"))),
        }
    });
    let value = expression_parser();
    target.then_ignore(just(Token::Special('='))).then(value).map(|(target, value)| super::Assignment { target, value })
}

fn statement_parser() -> impl Parser<Token, super::Statement, Error = Simple<Token>> {
    choice((
            assignment_parser().map(super::Statement::Assignment),
            expression_parser().map(super::Statement::Expression),
            ))
}


fn file_parser() -> impl Parser<Token, super::File, Error = Simple<Token>> {
    statement_parser().repeated().map(|statements| super::File { statements })
}

fn interactive_parser() -> impl Parser<Token, super::Interactive, Error = Simple<Token>> {
    statement_parser().map(|statement| super::Interactive { statement, gave_statement: false})
}


pub fn parse_file(input: &str) -> Result<super::File, ()> {
    file_parser().parse(tokenizer().parse(input).unwrap()).map(|value| value).map_err(|_| ())
}

pub fn parse_interactive(input: &str) -> Result<super::Interactive, ()> {
    interactive_parser().parse(tokenizer().parse(input).unwrap()).map(|value| value).map_err(|_| ())
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
    
    /*#[test]
    fn test_parse_float() {
        assert_eq!(parse("123.456"), Ok(Literal::Float(123.456)));
        assert_eq!(parse("-123.456"), Ok(Literal::Float(-123.456)));
    }*/

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
    
    #[test]
    fn test_command_name_parser() {
        let input = "python python/test.py \"hello\" \"hello\"";
        expression_parser().parse(tokenizer().parse(input).unwrap()).unwrap();
        panic!();
    }
}


