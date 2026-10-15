use std::{cell::RefCell, rc::Rc};

use crate::{env::LispEnv, lexer::tokenize, parser::parse};

pub mod env;
pub mod lexer;
pub mod parser;

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    String(String),                                   // String type
    Lambda(Box<LispLambda>),                          // user-difined function
    Number(f64),                                      // number type
    Symbol(String),                                   // for symbol
    List(Vec<LispExp>),                               // imply nested list
    Func(fn(&[LispExp]) -> Result<LispExp, LispErr>), // function type
    Bool(bool),                                       // boolean values (#t, #f)
    Nil,                                              // empty value (nil)
}

/// Lambda expression (user-defined function)
#[derive(Clone, Debug, PartialEq)]
pub struct LispLambda {
    pub params: Vec<String>,       // parameter names, e.g.["x", "y"]
    pub body: Box<LispExp>,        // function body, use Box to avoid stackoverflow
    pub env: Rc<RefCell<LispEnv>>, // the env of lambda
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
    use super::*;
    use crate::{env::LispEnv, lexer::tokenize, parser::parse};

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

    #[test]
    fn test_define_then_use_in_calc() {
        let mut env = create_default_env();
        eval_str("(define x 22)", &mut env).unwrap();
        assert_eq!(eval_str("(+ x 5)", &mut env).unwrap(), number!(27f64));
    }

    #[test]
    fn test_create_lambda() -> Result<(), LispErr> {
        let mut env = create_default_env();
        match eval_str("(lambda (x) (* x x))", &mut env).unwrap() {
            LispExp::Lambda(_) => Ok(()),
            _ => Err(LispErr::Reason(
                "eval_str() failed to create the lambda".to_string(),
            )),
        }
    }

    #[test]
    fn test_lambda_call() {
        let mut env = create_default_env();
        let _ = eval_str("(define add (lambda (a b) (+ a b)))", &mut env).unwrap();
        assert_eq!(eval_str("(add 4 9)", &mut env).unwrap(), number!(13f64));
    }

    #[test]
    fn test_lambda_direct_call() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("((lambda (x) (* x x)) 10)", &mut env).unwrap(),
            number!(100f64)
        );
    }

    #[test]
    fn test_implementation_of_abs_function() {
        let mut env = create_default_env();
        let _ = eval_str(
            "(define abs (lambda (x) (if (>= x 0) x (- 0 x))))",
            &mut env,
        );
        assert_eq!(eval_str("(abs 10)", &mut env).unwrap(), number!(10f64));
        assert_eq!(eval_str("(abs -10)", &mut env).unwrap(), number!(10f64));
    }

    #[test]
    fn test_implementation_of_sum_to_function() {
        let mut env = create_default_env();
        let _ = eval_str(
            "(define sum-to (lambda (x) (if (= x 1) 1 (+ x (sum-to (- x 1))))))",
            &mut env,
        );
        assert_eq!(eval_str("(sum-to 5)", &mut env).unwrap(), number!(15f64));
        assert_eq!(eval_str("(sum-to 6)", &mut env).unwrap(), number!(21f64));
    }

    #[test]
    fn test_implementation_of_compose_function() {
        let mut env = create_default_env();
        let _ = eval_str(
            "(define compose (lambda (f g) (lambda (x) (f (g x)))))",
            &mut env,
        )
        .unwrap();

        assert_eq!(
            eval_str(
                "((compose (lambda (x) (* x 2)) (lambda (x) (* x 3))) 20)",
                &mut env
            )
            .unwrap(),
            number!(120f64)
        );
    }

    #[test]
    // TODO: [bug] Each function call will leak the call frame into the top-level env.
    fn test_a_strange_function() {
        let mut env = create_default_env();
        let expected_env = Rc::new(RefCell::new(env.clone()));
        assert_eq!(
            eval_str("((lambda (x) x) (lambda (x) x))", &mut env).unwrap(),
            LispExp::Lambda(Box::new(LispLambda {
                params: vec!["x".to_string()],
                body: Box::new(LispExp::Symbol("x".to_string())),
                env: expected_env,
            }))
        );
    }

    #[test]
    fn test_closure() {
        let mut env = create_default_env();
        eval_str(
            "(define make-addr (lambda (n) (lambda (x) (+ x n))))",
            &mut env,
        )
        .unwrap();
        eval_str("(define add5 (make-addr 5))", &mut env).unwrap();
        assert_eq!(eval_str("(add5 10)", &mut env).unwrap(), number!(15f64));
    }

    #[test]
    fn test_tail_call_optimization() {
        let mut env = create_default_env();
        eval_str(
            "(define loop (lambda (n) (if (= n 0) \"done\" (loop (- n 1)))))",
            &mut env,
        )
        .unwrap();
        let res = eval_str("(loop 10000)", &mut env).unwrap();
        assert_eq!(res, string!("done"));
    }
}

/// eval() is the "Evaluator"
#[allow(dead_code)]
fn eval(exp: &LispExp, env: &mut LispEnv) -> Result<LispExp, LispErr> {
    // 01 Initialize: clone the current expresstion, take ownership of env
    let mut current_exp = exp.clone();
    let mut current_env = std::mem::take(env);
    // 02 trampoline loop: each loop processes one expression
    loop {
        match &current_exp {
            // -- Self-evaluating type: return directly --
            LispExp::Number(_)
            | LispExp::Bool(_)
            | LispExp::Nil
            | LispExp::Func(_)
            | LispExp::String(_)
            | LispExp::Lambda(_) => {
                *env = current_env; // return the original environment
                return Ok(current_exp.clone());
            }
            // -- Symbols: look up environment --
            LispExp::Symbol(x) => {
                let res = current_env.get(x);
                *env = current_env; // return the original environment
                return res;
            }
            // -- Lists --
            LispExp::List(elements) => {
                if elements.is_empty() {
                    *env = current_env; // return the original environment
                    return Ok(LispExp::Nil);
                }
                // -- "if" (tail position optimization) --
                if let LispExp::Symbol(s) = &elements[0] {
                    // (if condition true_branch false_branch)
                    if s == "if" && elements.len() == 4 {
                        let cond = eval(&elements[1], &mut current_env)?;
                        let is_true = !matches!(cond, LispExp::Bool(false) | LispExp::Nil);
                        // TCO!!!! do not recursively eval, update expression then continue
                        current_exp = if is_true {
                            elements[2].clone()
                        } else {
                            elements[3].clone()
                        };
                        continue;
                    }
                    // -- "define" (supports  recursive definition) --
                    else if s == "define" && elements.len() == 3 {
                        // use "Rc" to share environment: let closures "sees" themselves
                        if let LispExp::Symbol(name) = &elements[1] {
                            let share_env = Rc::new(RefCell::new(std::mem::take(&mut current_env)));
                            // first place a placeholder (will be replace with real value)
                            share_env.borrow_mut().set(name.clone(), LispExp::Nil);
                            // current_env takes shared_env as outer
                            current_env = LispEnv::with_outer(share_env.clone());
                            // evaluate (lambda's env shares the same "Rc" through outer)
                            let value = eval(&elements[2], &mut current_env)?;
                            // [KEY!!!] eval's internal TCO may have modified current_env
                            // must rebuild current_env pointer to shared_env
                            current_env = LispEnv::with_outer(share_env.clone());
                            // replace placeholder with real value
                            share_env.borrow_mut().set(name.clone(), value);
                            *env = current_env; // return the original environment
                            return Ok(LispExp::Nil);
                        } else {
                            *env = current_env; // return the original environment
                            return Err(LispErr::Reason(
                                "eval(): define needs a \'Symbol\'".to_string(),
                            ));
                        }
                    }
                    // -- lambda --
                    else if s == "lambda" && elements.len() >= 3 {
                        let params: Vec<String> = match &elements[1] {
                            LispExp::List(param_list) => param_list
                                .iter()
                                .map(|param| {
                                    if let LispExp::Symbol(x) = param {
                                        x.clone()
                                    } else {
                                        "?".to_string() // return "?" if param_list's element is not a symbol
                                    }
                                })
                                .collect(),
                            _ => {
                                *env = current_env;
                                return Err(LispErr::Reason(
                                    "eval(): lambda\'s parament must be a list".to_string(),
                                ));
                            }
                        };
                        let body = elements[2].clone();
                        let lambda = LispExp::Lambda(Box::new(LispLambda {
                            params: params,
                            body: Box::new(body),
                            // Lambda.env points to the definition-time CallFrame
                            env: Rc::new(RefCell::new(current_env.clone())),
                        }));
                        *env = current_env;
                        return Ok(lambda);
                    }
                }
                // -- Ordinary function call --
                let func = eval(&elements[0], &mut current_env)?;
                let args: Result<Vec<LispExp>, LispErr> = elements[1..]
                    .iter()
                    .map(|arg| eval(arg, &mut current_env))
                    .collect();
                match func {
                    LispExp::Func(f) => {
                        *env = current_env;
                        return f(&(args?));
                    }
                    LispExp::Lambda(lambda) => {
                        let mut new_env: LispEnv = LispEnv::with_outer(lambda.env.clone());
                        // bind parameter names to argument values one by one(set them in cloned environment)
                        for (param, arg) in lambda.params.iter().zip(args?.iter()) {
                            new_env.set(param.clone(), arg.clone());
                        }
                        // TCO! Update expression and environment, continue
                        current_exp = lambda.body.as_ref().clone();
                        current_env = new_env;
                        continue;
                    }
                    _ => {
                        *env = current_env;
                        return Err(LispErr::Reason("Not a function".to_string()));
                    }
                }
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
