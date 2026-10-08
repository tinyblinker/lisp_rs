pub mod lexer;
pub mod parser;

#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    Number(f64),    // number type
    String(String), // str type
    Symbol(String), // for symbol
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
        let exp_1 = LispExp::String(" 36 ".to_string());
        let exp_2 = LispExp::String("78".to_string());
        assert_eq!(LispExp::Number(36f64), eval_str(&exp_1).unwrap());
        assert_eq!(LispExp::Number(78f64), eval_str(&exp_2).unwrap());
    }

    #[test]
    fn test_eval_negative_num_in_str() {
        let exp_1 = LispExp::String(" -36 ".to_string());
        let exp_2 = LispExp::String("-78".to_string());
        assert_eq!(LispExp::Number(-36f64), eval_str(&exp_1).unwrap());
        assert_eq!(LispExp::Number(-78f64), eval_str(&exp_2).unwrap());
    }

    #[test]
    fn test_eval_fake_bool_str() {
        let exp_1 = LispExp::String("true".to_string());
        let exp_1_p = LispExp::String(" true ".to_string());
        let exp_2 = LispExp::String("false".to_string());
        let exp_2_p = LispExp::String("false ".to_string());
        assert_eq!(
            LispExp::String("true".to_string()),
            eval_str(&exp_1).unwrap()
        );
        assert_eq!(
            LispExp::String("true".to_string()),
            eval_str(&exp_1_p).unwrap()
        );
        assert_eq!(
            LispExp::String("false".to_string()),
            eval_str(&exp_2).unwrap()
        );
        assert_eq!(
            LispExp::String("false".to_string()),
            eval_str(&exp_2_p).unwrap()
        );
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
}

#[allow(dead_code)]
fn eval(exp: &LispExp) -> Result<LispExp, LispErr> {
    match *exp {
        LispExp::Number(x) => Ok(LispExp::Number(x)),
        _ => Err(LispErr::Reason("Unsupport in eval()".to_string())),
    }
}

#[allow(dead_code)]
fn eval_str(exp: &LispExp) -> Result<LispExp, LispErr> {
    match exp {
        LispExp::String(x) => {
            match x.trim() {
                "true" | "false" => return Ok(LispExp::String(x.trim().to_string())),
                _ => {}
            }
            let result: f64 = x
                .trim()
                .parse()
                .map_err(|_| LispErr::Reason("eval_str():not a num str".to_string()))?;
            Ok(LispExp::Number(result))
        }
        _ => Err(LispErr::Reason("eval_str():not a str".to_string())),
    }
}
