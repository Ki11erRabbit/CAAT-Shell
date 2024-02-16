
use crate::parser::{Literal, Expression};

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Float(f64),
    Integer(i64),
    String(String),
    Dollar,
    Bool(bool),
    Pipe,
    And,
    Or,
    Then,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Comma,
    Colon,
    ParenOpen,
    ParenClose,
}




peg::parser!{
    grammar parser() for str {
        pub rule identifier() -> Token
            = match_str:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']*) {?
                match match_str {
                    "true" => Ok(Token::Bool(true)),
                    "false" => Ok(Token::Bool(false)),
                    _ => Ok(Token::Identifier(match_str.to_string())),
                }
            }
        pub rule bool() -> Token
            = match_str:$("true" / "false") {Token::Bool(match_str.parse().unwrap())}
        pub rule float() -> Token
            = match_str:$(['0'..='9']+ ['.']['0'..='9']+) {Token::Float(match_str.parse().unwrap())}
        pub rule integer() -> Token
            = match_str:$(['-']?['0'..='9']+) {Token::Integer(match_str.parse().unwrap())}
        pub rule string() -> Token
            = ['"'] s:$([^ '"']+) ['"'] {Token::String(s.to_string())}
        pub rule pipe() -> Token
            = quiet!{"|"} {Token::Pipe}
        rule and() -> Token
            = quiet!{"&&"} {Token::And}
        rule or() -> Token
            = quiet!{"||"} {Token::Or}
        rule then() -> Token
            = quiet!{";"} {Token::Then}
        rule brace_open() -> Token
            = quiet!{"{"} {Token::BraceOpen}
        rule brace_close() -> Token
            = quiet!{"}"} {Token::BraceClose}
        rule bracket_open() -> Token
            = quiet!{"["} {Token::BracketOpen}
        rule bracket_close() -> Token
            = quiet!{"]"} {Token::BracketClose}
        rule comma() -> Token
            = quiet!{","} {Token::Comma}
        rule colon() -> Token
            = quiet!{":"} {Token::Colon}
        rule paren_open() -> Token
            = quiet!{"("} {Token::ParenOpen}
        rule paren_close() -> Token
            = quiet!{")"} {Token::ParenClose}
        rule dollar() -> Token
            = quiet!{"$"} {Token::Dollar}
        rule list() -> Literal
            = bracket_open() l:literal() ** comma() bracket_close() {Literal::List(l)}
        rule pair() -> Vec<(String, Literal)>
            = p:pair_item() ** comma() {p}
        rule pair_item() -> (String, Literal)
            = ['"'] k:$([^ '"']+) ['"'] colon() v:literal() { (k.to_string(), v) }
        rule map() -> Literal
            = brace_open() m:pair() brace_close() {Literal::Map(m)}
        rule base_literal() -> Literal 
            = token:(float() / integer() / string() / bool()) {
                match token {
                    Token::Float(f) => Literal::Float(f),
                    Token::Integer(i) => Literal::Integer(i),
                    Token::String(s) => Literal::String(s),
                    Token::Bool(b) => Literal::Boolean(b),
                    _ => unimplemented!(),
            }}
        rule literal() -> Literal
            = l:(base_literal() / list() / map()) {l}
        rule variable_expression() -> Expression
            = dollar() id:identifier() {
                match id {
                    Token::Identifier(s) => Expression::Variable(s),
                    _ => unimplemented!(),
                }
            }
        rule literal_expression() -> Expression
            = l:literal() {Expression::Literal(l)}
        rule paren_expression() -> Expression
            = paren_open() e:expression() paren_close() {Expression::Parenthesized(Box::new(e))}
        pub rule expression() -> Expression
            = e:(variable_expression() / literal_expression() / paren_expression()) {e}
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier() {
        assert_eq!(parser::identifier("foo"), Ok(Token::Identifier("foo".to_string())));
        assert_eq!(parser::identifier("true"), Ok(Token::Bool(true)));
        assert_eq!(parser::identifier("false"), Ok(Token::Bool(false)));
    }
    
    #[test]
    fn test_float() {
        assert_eq!(parser::float("3.14"), Ok(Token::Float(3.14)));
    }
    
    #[test]
    fn test_integer() {
        assert_eq!(parser::integer("42"), Ok(Token::Integer(42)));
        assert_eq!(parser::integer("-42"), Ok(Token::Integer(-42)));
    }

    #[test]
    fn test_string() {
        assert_eq!(parser::string(r#""foo""#), Ok(Token::String("foo".to_string())));
    }
    
    #[test]
    fn test_pipe() {
        assert_eq!(parser::pipe("|"), Ok(Token::Pipe));
    }

    #[test]
    fn test_expression() {
        assert_eq!(parser::expression("$foo"), Ok(Expression::Variable("foo".to_string())));
        assert_eq!(parser::expression("42"), Ok(Expression::Literal(Literal::Integer(42))));
        assert_eq!(parser::expression(r#""foo""#), Ok(Expression::Literal(Literal::String("foo".to_string()))));
        assert_eq!(parser::expression("[1, 2, 3]"), Ok(Expression::Literal(Literal::List(vec![Literal::Integer(1), Literal::Integer(2), Literal::Integer(3)]))));
        assert_eq!(parser::expression(r#"{"foo": "bar"}"#), Ok(Expression::Literal(Literal::Map(vec![("foo".to_string(), Literal::String("bar".to_string()))]))));
        assert_eq!(parser::expression(r#"(42)"#), Ok(Expression::Parenthesized(Box::new(Expression::Literal(Literal::Integer(42))))));
    }

}
