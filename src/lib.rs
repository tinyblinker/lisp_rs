use std::{cell::RefCell, rc::Rc};

use crate::{env::LispEnv, interner::intern, interpreter::eval, lexer::tokenize, parser::parse};

pub mod env;
pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    String(String),                                   // String type
    Lambda(Box<LispLambda>),                          // user-difined function
    Number(f64),                                      // number type
    Symbol(u64),                                      // for symbol
    List(Vec<LispExp>),                               // imply nested list
    Func(fn(&[LispExp]) -> Result<LispExp, LispErr>), // function type
    Bool(bool),                                       // boolean values (#t, #f)
    Nil,                                              // empty value (nil)
}

/// Lambda expression (user-defined function)
#[derive(Clone, Debug, PartialEq)]
pub struct LispLambda {
    pub params: Vec<u64>,          // parameter names, e.g.["x", "y"]
    pub body: Box<LispExp>,        // function body, use Box to avoid stackoverflow
    pub env: Rc<RefCell<LispEnv>>, // the env of lambda
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! symbol {
    ($name:expr) => {
        LispExp::Symbol($name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        env::LispEnv,
        interpreter::{create_default_env, eval},
        lexer::tokenize,
        parser::{parse, parse_atom},
    };

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
        let tokens = vec!["+"];
        let (exp, _) = parse(&tokens).unwrap();
        assert_eq!(exp, LispExp::Symbol(intern("+")));
    }

    #[test]
    fn test_parse_number_atom() {
        let tokens = vec!["32"];
        let (exp, _) = parse(&tokens).unwrap();
        assert_eq!(exp, LispExp::Number(32f64));
    }

    #[test]
    fn test_eval_str_source_code() {
        let mut env = LispEnv::new();
        let result = eval_str("+", &mut env);
        assert_eq!(result, eval(&LispExp::Symbol(intern("+")), &mut env));
    }

    #[test]
    fn test_recursive_parser() {
        let tokens = [
            "(".into(),
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
                symbol!(intern("+")),
                number!(1f64),
                list!(vec![symbol!(intern("*")), number!(2f64), number!(3f64)])
            ])
        );
    }

    #[test]
    fn test_unclosed_list_error() {
        assert_eq!(
            parse(&["(".into(), "+".into(), "1".into()]),
            Err(LispErr::Reason("Missing \')\'".to_string()))
        );
    }

    #[test]
    fn test_unexpected_closed_list_error() {
        assert_eq!(
            parse(&[")".into()]),
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
        env.set(intern("a"), number!(100f32));
        assert_eq!(env.get(intern("a")).unwrap(), number!(100f32));
    }

    #[test]
    fn test_lispenv_undefined_error() {
        let env = LispEnv::new();
        let var = "a";
        assert_eq!(
            env.get(intern(var)),
            Err(LispErr::Reason(format!("Undefined variable: {}", var)))
        )
    }

    #[test]
    fn test_eval_symbol() {
        let mut env = LispEnv::new();
        env.set(intern("x"), number!(42f64));
        assert_eq!(eval_str("x", &mut env).unwrap(), number!(42f64));
    }

    #[test]
    fn test_set_get_pi() {
        let mut env = LispEnv::new();
        env.set(intern("pi"), number!(3.14159));
        assert_eq!(eval_str("pi", &mut env).unwrap(), number!(3.14159))
    }

    /// the same key set multiple times will let the last one to overwrite
    /// the first one
    #[test]
    fn test_set_a_key_twice() {
        let mut env = LispEnv::new();
        env.set(intern("a"), number!(3));
        env.set(intern("a"), number!(2));
        assert_eq!(env.get(intern("a")).unwrap(), number!(2));
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
                params: vec![intern("x")],
                body: Box::new(LispExp::Symbol(intern("x"))),
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

    #[test]
    fn test_intern_same_name_gets_same_id() {
        let s1 = format!("intern-{}", "demo");
        let s2 = ["intern-", "demo"].concat();
        assert_ne!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(intern(&s1), intern(&s2));
        assert_eq!(intern(&s1), intern("intern-demo"));
    }

    #[test]
    fn test_intern_distinct_names_get_distinct_ids() {
        assert_ne!(intern("interning-name-a"), intern("interning-name-b"));
    }

    #[test]
    fn test_parse_atom_reuses_symbol_id() {
        assert_eq!(parse_atom("foo"), parse_atom("foo"));
        assert_eq!(parse_atom("foo"), LispExp::Symbol(intern("foo")));
    }

    #[test]
    fn test_ast_shares_symbol_id_within_expression() {
        let tokens = tokenize("(+ foo foo)");
        let (exp, _) = parse(&tokens).unwrap();
        match &exp {
            LispExp::List(elements) => match (&elements[1], &elements[2]) {
                (LispExp::Symbol(foo1), LispExp::Symbol(foo2)) => {
                    assert_eq!(foo1, foo2);
                    assert_eq!(*foo1, intern("foo"));
                }
                _ => panic!("expected two symbols in (+ foo foo)"),
            },
            _ => panic!("expected a list expression"),
        }
    }

    #[test]
    fn test_tokenize_empty() {
        let result = tokenize("");
        assert!(
            result.is_empty(),
            "empty string should return empty token list, not panic"
        )
    }

    #[test]
    fn test_tokenize_comment() {
        assert_eq!(
            tokenize("(+ 1 2) ; this is the comment"),
            vec!["(", "+", "1", "2", ")"]
        );
    }

    #[test]
    fn test_tokenize_string_literal() {
        assert_eq!(tokenize("\"hello\""), vec!["\"hello\""]);
    }

    #[test]
    fn test_begin() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(begin 1 2 3)", &mut env).unwrap(), number!(3));
        assert_eq!(eval_str("(begin)", &mut env).unwrap(), nil!());
    }

    #[test]
    fn test_set_bang() {
        let mut env = create_default_env();
        eval_str("(define x 10)", &mut env).unwrap();
        assert_eq!(eval_str("x", &mut env).unwrap(), number!(10));
        eval_str("(set! x 20)", &mut env).unwrap();
        assert_eq!(eval_str("x", &mut env).unwrap(), number!(20));
    }

    #[test]
    fn test_let() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(let ((x 1) (y 2)) (+ x y))", &mut env).unwrap(),
            number!(3)
        );
        assert_eq!(eval_str("(let () 42)", &mut env).unwrap(), number!(42));
    }

    #[test]
    fn test_cond() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(cond ((> 3 5) 1) ((< 3 5) 2) (else 3))", &mut env).unwrap(),
            number!(2)
        );
        // no match return nil
        assert_eq!(eval_str("(cond ((> 3 5) 1))", &mut env).unwrap(), nil!());
    }

    #[test]
    fn test_and() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(and #t 42)", &mut env).unwrap(), number!(42));
        assert_eq!(eval_str("(and #t #f)", &mut env).unwrap(), bool!(false));
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
