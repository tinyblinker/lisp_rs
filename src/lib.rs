pub mod lexer;

#[derive(Clone, Debug, PartialEq)]
enum LispExp {
    Number(f64),
    Symbol(String),
    Bool(bool),
    Nil(()),
}

#[derive(Clone, Debug, PartialEq)]
enum LispErr {
    Reason(String),
}

pub fn eval(exp: &LispExp) -> Result<LispExp, LispErr> {
    match exp {
        LispExp::Number(n) => Ok(LispExp::Number(*n)),
        _ => Err(LispErr::Reason(format! {"eval failed, src={:?}",exp})),
    }
}

fn eval_str(source: &str) -> Result<LispExp, LispErr> {
    let trimmed = source.trim();
    if let Ok(num) = trimmed.parse::<f64>() {
        eval(&LispExp::Number(num))
    } else if trimmed == "true" || trimmed == "false" {
        Ok(LispExp::Symbol(format! {"{}",trimmed}))
    } else {
        Err(LispErr::Reason(format! {"Not a number: {}", trimmed}))
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::tokenize;

    use super::*;
    #[test]
    fn test_create_number() {
        let n = LispExp::Number(21.0);
        assert_eq!(n, LispExp::Number(21.0));
    }
    #[test]
    fn test_eval_number() {
        let exp = LispExp::Number(21.0);
        let result = eval(&exp).unwrap();
        assert_eq!(result, LispExp::Number(21.0));
    }
    #[test]
    fn test_eval_str_number() {
        assert_eq!(eval_str("42").unwrap(), LispExp::Number(42.0))
    }
    #[test]
    fn test_eval_bool_str() {
        assert_eq!(eval_str("true").unwrap(), LispExp::Symbol(format! {"true"}));
        assert_eq!(
            eval_str("false").unwrap(),
            LispExp::Symbol(format! {"false"})
        );
    }
    #[test]
    fn test_tokenize_basic() {
        assert_eq!(tokenize("+ 1 4"), ["+", "1", "4"]);
    }
    #[test]
    fn test_tokenize_whitespace() {
        assert_eq!(tokenize("+  1   2"), ["+", "1", "2"])
    }
    #[test]
    fn test_tokenize_parens() {
        assert_eq!(tokenize("(+ 1 2)"), ["(", "+", "1", "2", ")"])
    }
}
