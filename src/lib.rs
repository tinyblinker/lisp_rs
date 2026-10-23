use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    env::LispEnv,
    interner::{intern, lookup},
    interpreter::eval,
    lexer::tokenize,
    parser::parse,
};

pub mod debug_helper;
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
    Macro(Box<LispLambda>),                           // Macro
    Number(f64),                                      // number type
    Symbol(u64),                                      // for symbol
    List(Vec<LispExp>),                               // imply nested list
    Func(fn(&[LispExp]) -> Result<LispExp, LispErr>), // function type
    Bool(bool),                                       // boolean values (#t, #f)
    Nil,                                              // empty value (nil)
}

/// we shall implement "Disply Trait" for LispExp
impl fmt::Display for LispExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LispExp::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            LispExp::Symbol(id) => {
                write!(f, "{}", lookup(*id))
            }
            LispExp::List(els) => {
                write!(f, "(")?;
                for (i, e) in els.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            LispExp::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            LispExp::Nil => write!(f, "nil"),
            LispExp::String(s) => write!(f, "\"{}\"", s),
            LispExp::Func(_) => write!(f, "#<builtin-function>"),
            LispExp::Lambda(lam) => {
                write!(
                    f,
                    "#<lambda ({} . {})> {}",
                    lam.params
                        .iter()
                        .map(|&id| lookup(id))
                        .collect::<Vec<_>>()
                        .join(" "),
                    match lam.rest {
                        Some(rest_param) => lookup(rest_param),
                        None => "None".to_string(),
                    },
                    lam.body
                )
            }
            LispExp::Macro(lam) => {
                write!(
                    f,
                    "#<macro ({} . {})> {}",
                    lam.params
                        .iter()
                        .map(|&id| lookup(id))
                        .collect::<Vec<_>>()
                        .join(" "),
                    match lam.rest {
                        Some(rest_param) => lookup(rest_param),
                        None => "None".to_string(),
                    },
                    lam.body
                )
            }
        }
    }
}

/// Lambda expression (user-defined function)
#[derive(Clone, Debug, PartialEq)]
pub struct LispLambda {
    pub params: Vec<u64>,          // parameter names, e.g.["x", "y"]
    pub body: Box<LispExp>,        // function body, use Box to avoid stackoverflow
    pub env: Rc<RefCell<LispEnv>>, // the env of lambda
    pub rest: Option<u64>,         // id of the rest parameters
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
        interpreter::{create_default_env, display_core_helper, eval},
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
                rest: None
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
        assert_eq!(eval_str("(and #t #t 42)", &mut env).unwrap(), number!(42));
        assert_eq!(eval_str("(and )", &mut env).unwrap(), bool!(true));
    }

    #[test]
    fn test_or() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(or #f #f 42)", &mut env).unwrap(), number!(42));
        assert_eq!(eval_str("(or #f #f #t)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(or )", &mut env).unwrap(), bool!(false));
    }

    #[test]
    fn test_let_star() {
        let mut env = create_default_env();
        // y can see x: this is the key difference from let
        assert_eq!(
            eval_str("(let* ((x 1) (y (+ x 1))) (+ x y))", &mut env).unwrap(),
            number!(3)
        );
        // three-level sequential binding
        assert_eq!(
            eval_str("(let* ((a 1) (b (+ a 1)) (c (+ b 1))) c)", &mut env).unwrap(),
            number!(3)
        );
    }

    #[test]
    fn test_letrec() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str(
                "(letrec ((even? (lambda (n) (if (= n 0) #t (odd? (- n 1)))))
                         (odd? (lambda (n) (if (= n 0) #f (even? (- n 1))))))
                 (even? 10))",
                &mut env
            )
            .unwrap(),
            bool!(true)
        );
    }

    #[test]
    fn test_letrec_nil() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str(
                "(let ((x 42))
                  (letrec ((y x)
                          (x 1))
                 y))",
                &mut env
            )
            .unwrap(),
            nil!()
        );
    }

    #[test]
    fn test_not() {
        // only #f and nil are "false"
        let mut env = create_default_env();
        assert_eq!(eval_str("(not #t)", &mut env).unwrap(), bool!(false));
        assert_eq!(eval_str("(not #f)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(not nil)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(not 0)", &mut env).unwrap(), bool!(false));
    }

    #[test]
    fn test_list() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(list 1 2 3)", &mut env).unwrap(),
            list!(vec![number!(1), number!(2), number!(3)])
        );
    }

    #[test]
    fn test_cons() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(cons 0 (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(0), number!(1), number!(2), number!(3)])
        );
    }

    #[test]
    fn test_car_cdr() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(car (list 1 2 3))", &mut env).unwrap(),
            number!(1)
        );
        assert_eq!(
            eval_str("(car )", &mut env),
            Err(LispErr::Reason("\'car\' have no arg".into()))
        );
        assert_eq!(
            eval_str("(car (list))", &mut env),
            Err(LispErr::Reason(
                "\'car\' cannot receives empty \'LispExp::List\' as the arg".into(),
            ))
        );
        assert_eq!(
            eval_str("(car nil)", &mut env),
            Err(LispErr::Reason(
                "\'car\' only receives \'LispExp::List\' as the arg".into(),
            ))
        );
        // ----------------------------------------------
        assert_eq!(
            eval_str("(cdr (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(2), number!(3)])
        );
        assert_eq!(
            eval_str("(cdr )", &mut env),
            Err(LispErr::Reason("\'cdr\' have no arg".into()))
        );
        assert_eq!(
            eval_str("(cdr (list))", &mut env),
            Err(LispErr::Reason(
                "\'cdr\' cannot receives empty \'LispExp::List\' as the arg".into(),
            ))
        );
        assert_eq!(
            eval_str("(cdr nil)", &mut env),
            Err(LispErr::Reason(
                "\'cdr\' only receives \'LispExp::List\' as the arg".into(),
            )),
        );
    }

    #[test]
    fn test_cadr_caddr() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(cadr (list 1 2 3))", &mut env).unwrap(),
            number!(2)
        );
        assert_eq!(
            eval_str("(cadr )", &mut env),
            Err(LispErr::Reason("\'cadr\' have no arg".into()))
        );
        assert_eq!(
            eval_str("(cadr (list))", &mut env),
            Err(LispErr::Reason(
                "\'cadr\' cannot receives empty \'LispExp::List\' as the arg".into(),
            ))
        );
        assert_eq!(
            eval_str("(cadr nil)", &mut env),
            Err(LispErr::Reason(
                "\'cadr\' only receives \'LispExp::List\' as the arg".into(),
            )),
        );
        // -----------------------------------------------------
        assert_eq!(
            eval_str("(caddr (list 1 2 3))", &mut env).unwrap(),
            number!(3)
        );
        assert_eq!(
            eval_str("(caddr )", &mut env),
            Err(LispErr::Reason("\'caddr\' have no arg".into()))
        );
        assert_eq!(
            eval_str("(caddr (list))", &mut env),
            Err(LispErr::Reason(
                "\'caddr\' cannot receives empty \'LispExp::List\' as the arg".into(),
            ))
        );
        assert_eq!(
            eval_str("(caddr nil)", &mut env),
            Err(LispErr::Reason(
                "\'caddr\' only receives \'LispExp::List\' as the arg".into(),
            )),
        );
    }

    #[test]
    fn test_append() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(append (list 1) (list 2 3))", &mut env).unwrap(),
            list!(vec![number!(1), number!(2), number!(3)])
        );
        assert_eq!(
            eval_str("(append nil (list 1))", &mut env).unwrap(),
            list!(vec![number!(1)])
        );
    }

    #[test]
    fn test_length() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(length (list 1 2 3))", &mut env).unwrap(),
            number!(3)
        );
        assert_eq!(eval_str("(length nil)", &mut env).unwrap(), number!(0));
        assert_eq!(eval_str("(length (list))", &mut env).unwrap(), number!(0));
    }

    #[test]
    fn test_reverse() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(reverse (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(3), number!(2), number!(1)])
        );
        assert_eq!(eval_str("(reverse nil)", &mut env).unwrap(), LispExp::Nil);
    }

    #[test]
    fn test_member() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(member 2 (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(2), number!(3)])
        );
        assert_eq!(
            eval_str("(member 5 (list 1 2 3))", &mut env).unwrap(),
            bool!(false)
        );
    }

    #[test]
    fn test_null_check() {
        let mut env = create_default_env();
        // only 'nil' is null
        assert_eq!(eval_str("(null? nil)", &mut env).unwrap(), bool!(true));
        assert_eq!(eval_str("(null? 0)", &mut env).unwrap(), bool!(false));
    }

    #[test]
    fn test_map() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(map (lambda (x) (+ x 1)) (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(2), number!(3), number!(4)])
        );
        eval_str("(define my_foo (lambda (x) (* x 2)))", &mut env).unwrap();
        assert_eq!(
            eval_str("(map my_foo (list 1 2 3))", &mut env).unwrap(),
            list!(vec![number!(2), number!(4), number!(6)])
        );
    }

    #[test]
    fn test_type_predicates() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(null? nil)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(null? 0)", &mut env).unwrap(),
            LispExp::Bool(false)
        );
        assert_eq!(
            eval_str("(number? 42)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(number? \"hello\")", &mut env).unwrap(),
            LispExp::Bool(false)
        );
        assert_eq!(
            eval_str("(symbol? 'x)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(boolean? #t)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(string? \"hi\")", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(procedure? +)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(pair? (list 1))", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(pair? nil)", &mut env).unwrap(),
            LispExp::Bool(false)
        );
        assert_eq!(
            eval_str("(list? (list 1 2))", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(list? nil)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
    }

    #[test]
    fn test_eq_equal() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(eq? 'x 'x)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(eq? 1 1)", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(equal? (list 1 (list 2)) (list 1 (list 2)))", &mut env).unwrap(),
            LispExp::Bool(true)
        );
        assert_eq!(
            eval_str("(equal? (list 1 2) (list 1 3))", &mut env).unwrap(),
            LispExp::Bool(false)
        );
    }

    #[test]
    fn test_apply() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(apply + (list 1 2 3))", &mut env).unwrap(),
            number!(6)
        );
        assert_eq!(
            eval_str("(apply (lambda (x y) (* x y)) (list 3 4))", &mut env).unwrap(),
            number!(12)
        );
    }

    #[test]
    fn test_filter() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(filter (lambda (x) (> x 0)) (list -1 2 -3 4))", &mut env).unwrap(),
            list!(vec![number!(2), number!(4)])
        );
        assert_eq!(
            eval_str("(filter (lambda (x) #f) (list 1 2 3))", &mut env).unwrap(),
            list!(vec![])
        );
    }

    #[test]
    fn test_variadic_lambda() {
        let mut env = create_default_env();
        eval_str("(define my_foo (lambda (x . the_rest) the_rest))", &mut env).unwrap();
        assert_eq!(
            eval_str("(my_foo 114514 1 2 3)", &mut env).unwrap(),
            list!(vec![number!(1), number!(2), number!(3)])
        );
    }

    #[test]
    fn test_quote_abbreviation() {
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(quote x)", &mut env).unwrap(),
            symbol!(intern("x"))
        );
        assert_eq!(eval_str("'x", &mut env).unwrap(), symbol!(intern("x")));
    }

    #[test]
    fn test_defmacro_basic() {
        let mut env = create_default_env();
        eval_str("(defmacro twice (x) (list '+ x x))", &mut env).unwrap();
        assert_eq!(eval_str("(twice 5)", &mut env).unwrap(), number!(10));
    }

    #[test]
    fn test_defmacro_when() {
        let mut env = create_default_env();
        eval_str(
            "(defmacro when (condition . body) (list 'if condition (cons 'begin body) 'nil))",
            &mut env,
        )
        .unwrap();
        assert_eq!(eval_str("(when #t 42)", &mut env).unwrap(), number!(42));
        assert_eq!(eval_str("(when #f 42)", &mut env).unwrap(), nil!());
    }

    #[test]
    fn test_gensym_in_defmacro_to_avoid_var_crash() {
        // case 1:
        let mut env = create_default_env();
        eval_str("(define tmp 21)", &mut env).unwrap();
        eval_str(
            "(defmacro my-or (a b)
                     (list 'let (list (list 'tmp a))
                       (list 'if 'tmp 'tmp b)))",
            &mut env,
        )
        .unwrap();
        // this is a normal case
        assert_eq!(eval_str("(my-or #f 42)", &mut env).unwrap(), number!(42));
        // this is when 'gensym' should takes effects (vars name is conflict)
        assert_eq!(eval_str("(my-or #f tmp)", &mut env).unwrap(), bool!(false));

        // case 2: (lambda will leak vars, so create a new env)
        let mut env = create_default_env();
        // to let (a_macro tmp) got '42', defmacro as below
        eval_str("(define tmp 21)", &mut env).unwrap();
        eval_str(
            "(defmacro my-or (a b)
                     (let ((tmp (gensym)))
                     (list 'let (list (list tmp a))
                       (list 'if tmp tmp b))))",
            &mut env,
        )
        .unwrap();
        assert_eq!(eval_str("(my-or #f tmp)", &mut env).unwrap(), number!(21));
    }

    #[test]
    fn test_quasiquote() {
        // `42 → 42
        let mut env = create_default_env();
        assert_eq!(eval_str("`42", &mut env).unwrap(), LispExp::Number(42.0));

        // `(,x) → value of x (assuming x=10) → (10)
        let mut env = create_default_env();
        eval_str("(define x 10)", &mut env).unwrap();
        assert_eq!(
            eval_str("`(,x)", &mut env).unwrap(),
            LispExp::List(vec![LispExp::Number(10.0)])
        );

        // `(a ,b c) → (a 2 c) (b=2, rest quoted)
        let mut env = create_default_env();
        eval_str("(define b 2)", &mut env).unwrap();
        let r = eval_str("`(a ,b c)", &mut env).unwrap();
        assert_eq!(
            r,
            list!(vec!(symbol!(intern("a")), number!(2), symbol!(intern("c")))),
        );

        // `(a ,@b c) → (a 1 2 c) (b=(1 2), rest quoted)
        let mut env = create_default_env();
        eval_str("(define b (list 1 2))", &mut env).unwrap();
        let r = eval_str("`(a ,@b c)", &mut env).unwrap();
        assert_eq!(
            r,
            list!(vec!(
                symbol!(intern("a")),
                number!(1),
                number!(2),
                symbol!(intern("c"))
            )),
        );

        // defmacro for 'when', in comparision
        let mut env = create_default_env();
        eval_str(
            "(defmacro when_a (condition . body)
                     (list 'if condition (cons 'begin body) 'nil))",
            &mut env,
        )
        .unwrap();
        eval_str(
            "(defmacro when_b (condition . body)
                    `(if ,condition (begin ,@body) nil))",
            &mut env,
        )
        .unwrap();
        let r_a = eval_str("(when_a #t (+ 1 2))", &mut env).unwrap();
        assert_eq!(r_a, number!(3));
        let r_b = eval_str("(when_b #t (+ 1 3))", &mut env).unwrap();
        assert_eq!(r_b, number!(4));
    }

    #[test]
    fn test_error_function() {
        // error msg in String
        let mut env = create_default_env();
        assert_eq!(
            eval_str("(error \"do not eat emacs\")", &mut env),
            Err(LispErr::Reason("\"do not eat emacs\"".into()))
        );
        // error msg from 'macro' (nearly the same as 'lambda')
        let mut env = create_default_env();
        eval_str(
            "(defmacro when (condition . body)
                    `(if ,condition (begin ,@body) nil))",
            &mut env,
        )
        .unwrap();
        assert_eq!(
            eval_str("(error when)", &mut env),
            Err(LispErr::Reason("#<macro (condition . body)> (quasiquote (if (unquote condition) (begin (unquote-splicing body)) nil))".into()))
        );
    }

    #[test]
    fn test_display_returns_nil() {
        let mut env = create_default_env();
        // display returns nil (it has a side effect — printing to screen — but returns nil)
        assert_eq!(eval_str("(display 42)", &mut env).unwrap(), LispExp::Nil);
        assert_eq!(
            eval_str("(display \"hello\")", &mut env).unwrap(),
            LispExp::Nil
        );
    }

    #[test]
    fn test_newline_returns_nil() {
        let mut env = create_default_env();
        assert_eq!(eval_str("(newline)", &mut env).unwrap(), LispExp::Nil);
    }

    #[test]
    fn test_display_write_to_sink() {
        let mut out = Vec::new();
        display_core_helper(&mut out, &number!(42)).unwrap();
        display_core_helper(&mut out, &symbol!(intern("+"))).unwrap();
        display_core_helper(&mut out, &string!("hello")).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "42+\"hello\"");
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
