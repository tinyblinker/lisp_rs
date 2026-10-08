#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    Number(f64), // number type
}

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr {
    Reason(String), // error message
}

#[cfg(test)]
mod tests {
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
}

#[allow(dead_code)]
fn eval(exp: &LispExp) -> Result<LispExp, LispErr> {
    match *exp {
        LispExp::Number(x) => Ok(LispExp::Number(x)),
        //   _ => Err(LispErr::Reason("Unsupport in eval()".to_string())),
    }
}
