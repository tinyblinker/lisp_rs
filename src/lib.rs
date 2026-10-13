use crate::{env::LispEnv, lexer::tokenize, parser::parse};

pub mod env;
pub mod lexer;
pub mod parser;

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    String(String),                                   // String type
    Number(f64),                                      // number type
    Symbol(String),                                   // for symbol
    List(Vec<LispExp>),                               // imply nested list
    Func(fn(&[LispExp]) -> Result<LispExp, LispErr>), // function type
    Bool(bool),                                       // boolean values (#t, #f)
    Nil,                                              // empty value (nil)
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
    ($name:expr) => {
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

#[macro_export]
#[allow(unused_macros)]
macro_rules! bool {
    ($name:expr) => {
        LispExp::Bool($name.into())
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! nil {
    () => {
        LispExp::Nil
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! string {
    ($name:expr) => {
        LispExp::String($name.into())
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr {
    Reason(String), // error message
}

pub fn create_default_env() -> LispEnv {
    let mut env = LispEnv::new();
    // set "+" function
    env.set(
        "+".into(),
        LispExp::Func(|args| {
            let sum: f64 = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(n) = arg {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .sum();
            Ok(number!(sum))
        }),
    );
    // set "-" function
    env.set(
        "-".into(),
        LispExp::Func(|args| {
            let nums: Vec<f64> = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(n) = arg {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            if nums.len() == 1 {
                Ok(number!(-nums[0]))
            } else {
                Ok(number!(nums[0] - nums[1..].iter().sum::<f64>()))
            }
        }),
    );
    // set "*" function
    env.set(
        "*".into(),
        LispExp::Func(|args| {
            let result: f64 = args.iter().fold(1.0f64, |acc, arg| match arg {
                LispExp::Number(num) => num * acc,
                _ => acc,
            });
            Ok(number!(result))
        }),
    );
    // set "/" function
    env.set(
        "/".into(),
        LispExp::Func(|args| {
            let nums: Vec<f64> = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(num) = arg {
                        Some(*num)
                    } else {
                        None
                    }
                })
                .collect();
            if nums.len() == 1 {
                Ok(number!(1f64 / nums[0]))
            } else if nums[1..].contains(&0f64) {
                Err(LispErr::Reason(
                    "Cannot divide \'0\' in division".to_string(),
                ))
            } else {
                let result: f64 = nums[1..].iter().fold(nums[0], |acc, num| acc / num);
                Ok(number!(result))
            }
        }),
    );
    // square function
    env.set(
        "square".to_string(),
        LispExp::Func(|args| {
            if let LispExp::Number(num) = &args[0] {
                Ok(number!(num * num))
            } else {
                Err(LispErr::Reason("square() needs a number".to_string()))
            }
        }),
    );
    // "=" comparison
    env.set(
        "=".into(),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x == y))
            } else {
                Err(LispErr::Reason("\'=\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // ">" comparison
    env.set(
        ">".into(),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x > y))
            } else {
                Err(LispErr::Reason("\'>\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // ">=" comparison
    env.set(
        ">=".into(),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x >= y))
            } else {
                Err(LispErr::Reason(
                    "\'>=\' needs a pair of numbers".to_string(),
                ))
            }
        }),
    );
    // "<" comparison
    env.set(
        "<".into(),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x < y))
            } else {
                Err(LispErr::Reason("\'<\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // "<=" comparison
    env.set(
        "<=".into(),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x <= y))
            } else {
                Err(LispErr::Reason(
                    "\'<=\' needs a pair of numbers".to_string(),
                ))
            }
        }),
    );
    // "string-length" function
    env.set(
        "string-length".into(),
        LispExp::Func(|args| {
            if let LispExp::String(str) = &args[0] {
                Ok(LispExp::Number(str.len() as f64))
            } else {
                Err(LispErr::Reason(
                    "\'string-length function\' needs a \'String\'".to_string(),
                ))
            }
        }),
    );
    env
}

#[cfg(test)]
mod tests {

    use crate::{env::LispEnv, lexer::tokenize, parser::parse};

    use super::*;

    #[test]
    fn test_create_number() {
        let n = LispExp::Number(88.0);
        assert_eq!(n, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_number() {
        let mut env = LispEnv::new();
        let exp = LispExp::Number(88.0);
        let result = eval(&exp, &mut env).unwrap(); // eval() is essential,TDD to develop it
        assert_eq!(result, LispExp::Number(88.0));
    }

    #[test]
    fn test_eval_num_in_str() {
        let mut env = LispEnv::new();
        assert_eq!(LispExp::Number(36f64), eval_str(" 36 ", &mut env).unwrap());
        assert_eq!(LispExp::Number(78f64), eval_str("78", &mut env).unwrap());
    }

    #[test]
    fn test_eval_negative_num_in_str() {
        let mut env = LispEnv::new();
        assert_eq!(
            LispExp::Number(-36f64),
            eval_str(" -36 ", &mut env).unwrap()
        );
        assert_eq!(LispExp::Number(-78f64), eval_str("-78", &mut env).unwrap());
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
        let mut env = LispEnv::new();
        let result = eval_str("+", &mut env);
        assert_eq!(result, eval(&LispExp::Symbol("+".into()), &mut env));
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
    fn test_error_missing_parenthese() {
        let mut env = LispEnv::new();
        assert_eq!(
            eval_str("(+ 1 2", &mut env),
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
        assert_eq!(eval_str("x", &mut env).unwrap(), number!(42f64));
    }

    #[test]
    fn test_set_get_pi() {
        let mut env = LispEnv::new();
        env.set("pi".into(), number!(3.14159));
        assert_eq!(eval_str("pi", &mut env).unwrap(), number!(3.14159))
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

    #[test]
    fn test_eval_addition() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(+ 1 2)", &mut env).unwrap(), number!(3));
    }

    #[test]
    fn test_eval_subtraction() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(- 10 3)", &mut env).unwrap(), number!(7f64));
        assert_eq!(eval_str("(- 10 3 2)", &mut env).unwrap(), number!(5f64));
        assert_eq!(eval_str("(- 5)", &mut env).unwrap(), number!(-5f64));
    }

    #[test]
    fn test_eval_multiplication() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(* 10)", &mut env).unwrap(), number!(10f64));
        assert_eq!(eval_str("(* 10 3)", &mut env).unwrap(), number!(30f64));
        assert_eq!(eval_str("(* 10 2 3)", &mut env).unwrap(), number!(60f64));
    }

    #[test]
    fn test_eval_division() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(/ 10 2)", &mut env).unwrap(), number!(5f64));
        assert_eq!(eval_str("(/ 100 5 2)", &mut env).unwrap(), number!(10f64));
        assert_eq!(eval_str("(/ 10)", &mut env).unwrap(), number!(0.1f64));
    }

    #[test]
    fn test_eval_division_with_zero_error() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(/ 10 0)", &mut env),
            Err(LispErr::Reason(
                "Cannot divide \'0\' in division".to_string(),
            ))
        );
        assert_eq!(
            eval_str("(/ 100 5 2 23 0)", &mut env),
            Err(LispErr::Reason(
                "Cannot divide \'0\' in division".to_string(),
            ))
        );
    }

    #[test]
    fn test_eval_square() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(square 2)", &mut env).unwrap(), number!(4f64));
        assert_eq!(
            eval_str("(square +)", &mut env),
            Err(LispErr::Reason("square() needs a number".to_string()))
        )
    }

    #[test]
    fn test_eval_unary_minus() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(- 1)", &mut env).unwrap(), number!(-1f64));
    }

    #[test]
    fn test_eval_addition_multiple_numbers() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(+ 1 2 3 4 5)", &mut env).unwrap(), number!(15f64));
    }

    #[test]
    fn eval_more_complex_arithmetic() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(+ 1 2 3)", &mut env).unwrap(), number!(6f64));
        assert_eq!(eval_str("(+ (+ 1 2) 3)", &mut env).unwrap(), number!(6f64));
        assert_eq!(eval_str("(- 10 5 2)", &mut env).unwrap(), number!(3f64));
    }

    #[test]
    fn test_lisp_type_bool_nil() {
        let mut env = create_default_env();
        assert_eq!(eval_str("#t", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("#f", &mut env).unwrap(), bool!(false));
        assert_eq!(eval_str("nil", &mut env).unwrap(), nil!());
    }

    #[test]
    fn test_eval_lisp_comparison() {
        let mut env = create_default_env();
        // =
        assert_eq!(eval_str("(= 1 1)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(= 1 2)", &mut env).unwrap(), bool!(false));
        // >
        assert_eq!(eval_str("(> 1 3)", &mut env).unwrap(), bool!(false));
        assert_eq!(eval_str("(> 3 1)", &mut env).unwrap(), bool!(true));
        // >=
        assert_eq!(eval_str("(>= 1 3)", &mut env).unwrap(), bool!(false));
        assert_eq!(eval_str("(>= 3 3)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(>= 3 1)", &mut env).unwrap(), bool!(true));
        // <
        assert_eq!(eval_str("(< 1 3)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(< 3 1)", &mut env).unwrap(), bool!(false));
        // <=
        assert_eq!(eval_str("(<= 1 3)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(<= 3 3)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(<= 3 1)", &mut env).unwrap(), bool!(false));
    }

    #[test]
    fn test_eval_lisp_type_string() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("\"hello_lisp\"", &mut env).unwrap(),
            string!("hello_lisp")
        );
    }

    #[test]
    fn test_eval_string_length() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(string-length \"abcd\")", &mut env).unwrap(),
            number!(4f64)
        );
    }

    #[test]
    fn test_if_true_branch() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(if #t 1 2)", &mut env).unwrap(),
            LispExp::Number(1f64)
        );
    }

    #[test]
    fn test_if_false_branch() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(if #f 1 2)", &mut env).unwrap(),
            LispExp::Number(2f64)
        );
    }

    #[test]
    fn test_if_comparison() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(if (> 1 2) 1 2)", &mut env).unwrap(),
            LispExp::Number(2f64)
        );
    }

    #[test]
    fn test_define_variable_and_lookup_it() {
        let mut env = create_default_env();
        eval_str("(define x 10)", &mut env).unwrap();
        assert_eq!(eval_str("x", &mut env).unwrap(), number!(10f64));
    }
}

/// eval() is the "Evaluator"
#[allow(dead_code)]
fn eval(exp: &LispExp, mut env: &mut LispEnv) -> Result<LispExp, LispErr> {
    match exp {
        LispExp::Number(x) => Ok(LispExp::Number(*x)),
        LispExp::Symbol(x) => env.get(x), // symbol->look up in "env"
        LispExp::Bool(_) | LispExp::Nil | LispExp::Func(_) | LispExp::String(_) => Ok(exp.clone()), // function itself is a value
        LispExp::List(elements) => {
            // empty list
            if elements.is_empty() {
                return Ok(LispExp::List(vec![]));
            }
            // special treat for "if" and "define"
            if let LispExp::Symbol(s) = &elements[0] {
                if s == "if" && elements.len() == 4 {
                    let cond = eval(&elements[1], &mut env)?;
                    let is_true = !matches!(cond, LispExp::Bool(false) | LispExp::Nil);
                    return if is_true {
                        eval(&elements[2], &mut env)
                    } else {
                        eval(&elements[3], &mut env)
                    };
                } else if s == "define" && elements.len() == 3 {
                    let value = eval(&elements[2], &mut env).unwrap();
                    if let LispExp::Symbol(name) = &elements[1] {
                        env.set(name.to_string(), value);
                        return Ok(LispExp::Nil); // define itself return nil
                    } else {
                        return Err(LispErr::Reason(
                            "eval(): define needs a \'Symbol\'".to_string(),
                        ));
                    }
                }
            }
            // eval the first ATOM to get the function
            let func = eval(&elements[0], &mut env)?;
            let args: Result<Vec<LispExp>, LispErr> = elements[1..]
                .iter()
                .map(|arg| eval(arg, &mut env))
                .collect();
            match func {
                LispExp::Func(f) => f(&(args?)),
                _ => Err(LispErr::Reason("Not a function".to_string())),
            }
        }
    }
}

/// eval_str() is the pipeline
/// source_code-->Lexer(tokenize())->[tokens]->Parser(parse())->[AST]->Evaluator(eval())-->Value
#[allow(dead_code)]
fn eval_str(source: &str, mut env: &mut LispEnv) -> Result<LispExp, LispErr> {
    // "Lexer" to tokenize the source code to tokens
    let tokens = tokenize(source);
    // "Parser" to parse the tokens to AST
    let (exp, _) = parse(&tokens)?;
    // "Evaluator" to eval the AST
    eval(&exp, &mut env)
}
