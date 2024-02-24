
use crate::parser::{Literal, Expression, Command, Pipeline, Operator, Statement, Assignment, Interactive, File, FunctionDef, Redirect, PipelinePart};

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
    Concat,
    InputRedirect,
    OutputRedirect,
    AppendRedirect,
}




peg::parser!{
    grammar parser() for str {
        pub rule identifier() -> Token
            = match_str:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']*) {?
                match match_str {
                    "true" => Err("boolean not identifier"),
                    "false" => Err("boolean not identifier"),
                    "if" => Err("if not identifier"),
                    "then" => Err("then not identifier"),
                    "else" => Err("else not identifier"),
                    //"access" => Err("access not identifier"),
                    //"at" => Err("at not identifier"),
                    "function" => Err("function not identifier"),
                    "return" => Err("return not identifier"),
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
        rule concat() -> Token 
            = quiet!{"++"} {Token::Concat}
        rule input_redirect() -> Token
            = quiet!{"<"} {Token::InputRedirect}
        rule output_redirect() -> Token
            = quiet!{">"} {Token::OutputRedirect}
        rule append_redirect() -> Token
            = quiet!{">>"} {Token::AppendRedirect}
        rule redirect_input() -> Redirect
            = input_redirect() [' '|'\t']* e:expression() {Redirect::Input(Box::new(e))}
        rule redirect_output() -> Redirect
            = output_redirect() [' '|'\t']* e:expression() {Redirect::Output(Box::new(e))}
        rule redirect_append() -> Redirect
            = append_redirect() [' '|'\t']* e:expression() {Redirect::Append(Box::new(e))}
        rule list() -> Literal
            = bracket_open() [' '|'\t']* l:literal() ** (comma() [' '|'\t']*) bracket_close() {Literal::List(l)}
        rule pair() -> Vec<(String, Literal)>
            = p:pair_item() ** (comma() [' '|'\t']*) {p}
        rule pair_item() -> (String, Literal)
            = ['"'] k:$([^ '"']+) ['"'] ([' '|'\t']* colon() [' '|'\t']*) v:literal() { (k.to_string(), v) }
        rule map() -> Literal
            = brace_open() [' '|'\t']* m:pair() brace_close() {Literal::Map(m)}
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
        rule higher_order() -> Expression
            = brace_open() [' '|'\t']* p:pipeline() [' '|'\t']* brace_close() {Expression::HigherOrder(p)}
                   
        rule if_expression() -> Expression 
            = "if" [' '|'\t'|'\n']* cond:expression() [' '|'\t'|'\n']* "then" [' '|'\t'|'\n']* then:expression() [' '|'\t'|'\n']* "else" [' '|'\t'|'\n']* else_:expression() {
            Expression::If(Box::new(cond), Box::new(then), Box::new(else_))
            }
        #[cache_left_rec]
        rule access_expression() -> Expression 
            = thing:(expression_nonterminals_right() / expression_terminals() / concat_expression() /access_expression()) [' '|'\t']* bracket_open() [' '|'\t']* index:expression() [' '|'\t']* bracket_close() {
                Expression::Access(Box::new(thing), Box::new(index))
            }
        rule concat_expression() -> Expression
            = e1:(expression_terminals() / expression_nonterminals_right()) [' '|'\t']* concat() [' '|'\t']* e2:expression() {
            Expression::Concat(Box::new(e1), Box::new(e2))
            }
        rule expression_terminals() -> Expression
            = e:(variable_expression() / literal_expression() / higher_order() / pipeline_expression()) {e}
        rule expression_nonterminals_right() -> Expression
            = e:(if_expression() / paren_expression() / expression_terminals()) {e}
        rule expression_nonterminals() -> Expression
            = e:(if_expression() / access_expression() / concat_expression() / paren_expression() / expression_terminals()) {e}
        pub rule expression() -> Expression
            = e:(expression_nonterminals() / expression_terminals()) {e}
        pub rule command() -> Command
            = name:identifier() [' '|'\t']+ args:expression() ** ([' '|'\t']+) {
                if let Token::Identifier(name) = name {
                    Command::new(name, args)
                } else {
                    unimplemented!()
                }
            } / name:identifier() {
                if let Token::Identifier(name) = name {
                    Command::new(name, vec![])
                } else {
                    unimplemented!()
                }
            }
        rule operator() -> Operator
            = op:(pipe() / and() / or() / then()) {
                match op {
                    Token::Pipe => Operator::Pipe,
                    Token::And => Operator::And,
                    Token::Or => Operator::Or,
                    Token::Then => Operator::Then,
                    _ => unimplemented!(),
                }
                            
            }
        pub rule pipeline_part() -> PipelinePart
            = c:command() [' '|'\t']* o:operator() [' '|'\t']* n:pipeline_part() {
            PipelinePart{command: c, operator: Some(o), next: Some(Box::new(n))}
            }
            / c:command() {PipelinePart{command: c, operator: None, next: None}}
        pub rule pipeline() -> Pipeline
            = p:pipeline_part() [' '|'\t']* r:(redirect_input() / redirect_output() / redirect_append()) {Pipeline {pipeline: p, redirect: Some(r)}}
            / p:pipeline_part() {Pipeline {pipeline: p, redirect: None}}
        rule pipeline_expression() -> Expression
            = p:pipeline() {Expression::Pipeline(p)}
        rule expression_statement() -> Statement
            = e:expression() {Statement::Expression(e)} 
        rule assignment() -> Assignment
            = id:identifier() [' '|'\t']* ['='] [' '|'\t']* e:expression() {
                if let Token::Identifier(s) = id {
                    Assignment{target: s, value: e}
                } else {
                    unreachable!()
                }
            }
        rule assignment_statement() -> Statement
            = a:assignment() {Statement::Assignment(a)}
        rule function_def() -> FunctionDef
            = "function" [' '|'\t']* id:identifier() [' '|'\t']* ['('] [' '|'\t']* args:identifier() ** (comma() [' '|'\t']*) [' '|'\t']* [')'] [' '|'\t']* ['{'] [' '|'\t'|'\r'|'\n']* body:file() [' '|'\t']* ['}'] {
                if let Token::Identifier(name) = id {
                    let args = args.into_iter().map(|t| if let Token::Identifier(s) = t {s} else {unreachable!()}).collect();
                    FunctionDef { name: name, args: args, body: body }
                } else {
                    unreachable!()
                }
            }
        rule function_def_statement() -> Statement
            = f:function_def() {Statement::FunctionDef(f)}
        rule return_statement() -> Statement
            = "return" [' '|'\t']* e:expression() {Statement::Return(e)}
        rule comment() -> Statement
            = ['#'] c:$([^ '\n']+) ['\r']?['\n']* {Statement::Comment(c.to_string())}
        rule blank() -> Statement
            = [' '|'\t']* ['\r']?['\n']* {Statement::Blank}
        rule statement() -> Statement
            = [' '|'\t']* s:(assignment_statement() / expression_statement() / function_def_statement() / return_statement() / comment() / blank()) {s}
        pub rule interactive() -> Interactive
            = s:statement() ![_]{Interactive { statement: Some(s) }}
        pub rule file() -> File
            = s:statement() ** (['\r']?['\n']+) ['\r']?['\n']* {
                let mut statements = s.into_iter().filter(|s| match s {Statement::Blank => false, _ => true}).collect();
                File::new(statements)
            }
        pub rule shebang() -> String
            = "#!" s:$([^ '\n']+)  ['\r']?['\n']* [_]* ![_] {s.to_string()}
    }
    
}

pub use parser::file as parse_file;
pub use parser::interactive as parse_interactive;
pub use parser::shebang as parse_shebang;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier() {
        assert_eq!(parser::identifier("foo"), Ok(Token::Identifier("foo".to_string())));
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
    
    #[test]
    fn test_command() {
        assert_eq!(parser::command("foo 42"), Ok(Command::new("foo".to_string(), vec![Expression::Literal(Literal::Integer(42))])));
    }
    
    #[test]
    fn test_pipeline() {
        assert_eq!(parser::pipeline("foo 42"), Ok(Pipeline {pipeline: PipelinePart {command: Command::new("foo".to_string(), vec![Expression::Literal(Literal::Integer(42))]), operator: None, next: None}, redirect: None}));
    }

    #[test]
    fn test_pipeline_redirect() {
        assert_eq!(parser::pipeline("foo 42 > \"bar\""), Ok(Pipeline {pipeline: PipelinePart {command: Command::new("foo".to_string(), vec![Expression::Literal(Literal::Integer(42))]), operator: None, next: None}, redirect: Some(Redirect::Output(Box::new(Expression::Literal(Literal::String("bar".to_string())))))}));
    }
}
