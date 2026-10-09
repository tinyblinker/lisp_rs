use crate::{lexer::tokenize, parser::parse};

pub mod lexer;
pub mod parser;

#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    Number(f64),        // number type
    String(String),     // str type
    Symbol(String),     // for symbol
    List(Vec<LispExp>), // imply nested list
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! symbol {
    ($name:literal) => {
        LispExp::Symbol($name.into())
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! number {
    ($name:literal) => {
        LispExp::Number($name.into())
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! list {
    ($name:expr) => {
        LispExp::List($name.into())
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr {
    Reason(String), // error message
}

#[cfg(test)]
mod tests {

    use crate::{lexer::tokenize, parser::parse};

    use super::*;

    #[test]
    fn test_create_number() {
        let n = LispExp::Number(88.0);
        assert_eq!(n, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_number() {
        let exp = LispExp::Number(88.0);
        let result = eval(&exp).unwrap(); // eval() is essential,TDD to develop it
        assert_eq!(result, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_num_in_str() {
        assert_eq!(LispExp::Number(36f64), eval_str(" 36 ").unwrap());
        assert_eq!(LispExp::Number(78f64), eval_str("78").unwrap());
    }

    #[test]
    fn test_eval_negative_num_in_str() {
        assert_eq!(LispExp::Number(-36f64), eval_str(" -36 ").unwrap());
        assert_eq!(LispExp::Number(-78f64), eval_str("-78").unwrap());
    }

    #[test]
    fn test_eval_fake_bool_str() {
        assert_eq!(eval(&LispExp::String("true".into())), eval_str("true"));
        assert_eq!(eval(&LispExp::String("true".into())), eval_str("  true "));
        assert_eq!(eval(&LispExp::String("false".into())), eval_str("false"));
        assert_eq!(eval(&LispExp::String("false".into())), eval_str("  false "));
    }

    #[test]
    fn test_tokenize_multi() {
        assert_eq!(tokenize("+ 1 2"), ["+", "1", "2"])
    }

    #[test]
    fn test_tokenize_whitespace() {
        assert_eq!(tokenize(" +   1     2"), ["+", "1", "2"])
    }

    #[test]
    fn test_tokenize_parentheses() {
        assert_eq!(tokenize("(+ 1 2)"), ["(", "+", "1", "2", ")"])
    }

    #[test]
    fn test_tokenize_comments() {
        assert_eq!(
            tokenize("(+ 1 2) ; This is a comment"),
            ["(", "+", "1", "2", ")"]
        );
    }

    #[test]
    fn test_parse_symbol_atom() {
        let tokens = vec!["+".to_string()];
        let (exp, _) = parse(&tokens).unwrap();
        assert_eq!(exp, LispExp::Symbol("+".into()));
    }

    #[test]
    fn test_parse_number_atom() {
        let tokens = vec!["32".to_string()];
        let (exp, _) = parse(&tokens).unwrap();
        assert_eq!(exp, LispExp::Number(32f64));
    }

    #[test]
    fn test_eval_str_source_code() {
        let result = eval_str("+");
        assert_eq!(result, eval(&LispExp::Symbol("+".into())));
    }

    #[test]
    fn test_recursive_parser() {
        assert_eq!(
            eval_str("(+ 1 (* 2 3))").unwrap(),
            list!(vec![
                symbol!("+"),
                number!(1f64),
                list!(vec![symbol!("*"), number!(2f64), number!(3f64)])
            ])
        );
    }
}

/// eval() is the "Evaluator"
#[allow(dead_code)]
fn eval(exp: &LispExp) -> Result<LispExp, LispErr> {
    match exp {
        LispExp::Number(x) => Ok(LispExp::Number(*x)),
        LispExp::List(x) => Ok(LispExp::List(x.to_vec())),
        _ => Err(LispErr::Reason("Unsupport in eval()".to_string())),
    }
}

/// eval_str() is the pipeline
/// source_code-->Lexer(tokenize())->[tokens]->Parser(parse())->[AST]->Evaluator(eval())-->Value
#[allow(dead_code)]
fn eval_str(source: &str) -> Result<LispExp, LispErr> {
    // "Lexer" to tokenize the source code to tokens
    let tokens = tokenize(source);
    // "Parser" to parse the tokens to AST
    let (exp, _) = parse(&tokens)?;
    // "Evaluator" to eval the AST
    eval(&exp)
}
