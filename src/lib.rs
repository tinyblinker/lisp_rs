use crate::{env::LispEnv, lexer::tokenize, parser::parse};

pub mod env;
pub mod lexer;
pub mod parser;

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    Number(f64),                                      // number type
    Symbol(String),                                   // for symbol
    List(Vec<LispExp>),                               // imply nested list
    Func(fn(&[LispExp]) -> Result<LispExp, LispErr>), // function type
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

    use crate::{
        env::LispEnv,
        lexer::tokenize,
        parser::{parse, parse_atom},
    };

    use super::*;

    #[test]
    fn test_create_number() {
        let n = LispExp::Number(88.0);
        assert_eq!(n, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_number() {
        let env = LispEnv::new();
        let exp = LispExp::Number(88.0);
        let result = eval(&exp, &env).unwrap(); // eval() is essential,TDD to develop it
        assert_eq!(result, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_num_in_str() {
        let env = LispEnv::new();
        assert_eq!(LispExp::Number(36f64), eval_str(" 36 ", &env).unwrap());
        assert_eq!(LispExp::Number(78f64), eval_str("78", &env).unwrap());
    }

    #[test]
    fn test_eval_negative_num_in_str() {
        let env = LispEnv::new();
        assert_eq!(LispExp::Number(-36f64), eval_str(" -36 ", &env).unwrap());
        assert_eq!(LispExp::Number(-78f64), eval_str("-78", &env).unwrap());
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
        let env = LispEnv::new();
        let result = eval_str("+", &env);
        assert_eq!(result, eval(&LispExp::Symbol("+".into()), &env));
    }

    #[test]
    fn test_recursive_parser() {
        let tokens = [
            "(".to_string(),
            "+".into(),
            "1".into(),
            "(".into(),
            "*".into(),
            "2".into(),
            "3".into(),
            ")".into(),
            ")".into(),
        ];
        let (result, _) = parse(&tokens).unwrap();
        assert_eq!(
            result,
            list!(vec![
                symbol!("+"),
                number!(1f64),
                list!(vec![symbol!("*"), number!(2f64), number!(3f64)])
            ])
        );
    }

    #[test]
    fn test_unclosed_list_error() {
        assert_eq!(
            parse(&["(".to_string(), "+".into(), "1".into()]),
            Err(LispErr::Reason("Missing \')\'".to_string()))
        );
    }

    #[test]
    fn test_unexpected_closed_list_error() {
        assert_eq!(
            parse(&[")".to_string()]),
            Err(LispErr::Reason("Extra \')\'".to_string()))
        );
    }

    #[test]
    fn test_parse_atom_custom_symbols() {
        assert_eq!(parse_atom("#t"), symbol!("#t"));
        assert_eq!(parse_atom("#f"), symbol!("#f"));
    }

    #[test]
    fn test_error_missing_parenthese() {
        let env = LispEnv::new();
        assert_eq!(
            eval_str("(+ 1 2", &env),
            Err(LispErr::Reason("Missing \')\'".to_string()))
        );
    }

    /// hint: maybe just write the test first, just let test drive you to develop(TDD)
    #[test]
    fn test_lispenv_set_get() {
        let mut env = LispEnv::new();
        env.set("a".into(), number!(100f32));
        assert_eq!(env.get("a").unwrap(), number!(100f32));
    }

    #[test]
    fn test_lispenv_undefined_error() {
        let env = LispEnv::new();
        let var = "a";
        assert_eq!(
            env.get(var),
            Err(LispErr::Reason(format!("Undefined variable: {}", var)))
        )
    }

    #[test]
    fn test_eval_symbol() {
        let mut env = LispEnv::new();
        env.set("x".into(), number!(42f64));
        assert_eq!(eval_str("x", &env).unwrap(), number!(42f64));
    }

    #[test]
    fn test_set_get_pi() {
        let mut env = LispEnv::new();
        env.set("pi".into(), number!(3.14159));
        assert_eq!(eval_str("pi", &env).unwrap(), number!(3.14159))
    }

    /// the same key set multiple times will let the last one to overwrite
    /// the first one
    #[test]
    fn test_set_a_key_twice() {
        let mut env = LispEnv::new();
        env.set("a".into(), number!(3));
        env.set("a".into(), number!(2));
        assert_eq!(env.get("a").unwrap(), number!(2));
    }
}

/// eval() is the "Evaluator"
#[allow(dead_code)]
fn eval(exp: &LispExp, env: &LispEnv) -> Result<LispExp, LispErr> {
    match exp {
        LispExp::Number(x) => Ok(LispExp::Number(*x)),
        LispExp::Symbol(x) => env.get(x),
        LispExp::Func(_) => Ok(exp.clone()), // function itself is a value
        _ => Err(LispErr::Reason(
            "eval(): This type is not support yet".to_string(),
        )),
    }
}

/// eval_str() is the pipeline
/// source_code-->Lexer(tokenize())->[tokens]->Parser(parse())->[AST]->Evaluator(eval())-->Value
#[allow(dead_code)]
fn eval_str(source: &str, env: &LispEnv) -> Result<LispExp, LispErr> {
    // "Lexer" to tokenize the source code to tokens
    let tokens = tokenize(source);
    // "Parser" to parse the tokens to AST
    let (exp, _) = parse(&tokens)?;
    // "Evaluator" to eval the AST
    eval(&exp, env)
}
