#[derive(Clone, Debug, PartialEq)]
pub enum LispExp {
    Number(f64), // number type
    Str(String), // str type
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

    #[test]
    fn test_eval_num_in_str() {
        let exp_1 = LispExp::Str(" 36 ".to_string());
        let exp_2 = LispExp::Str("78".to_string());
        assert_eq!(LispExp::Number(36f64), eval_str(&exp_1).unwrap());
        assert_eq!(LispExp::Number(78f64), eval_str(&exp_2).unwrap());
    }

    #[test]
    fn test_eval_negative_num_in_str() {
        let exp_1 = LispExp::Str(" -36 ".to_string());
        let exp_2 = LispExp::Str("-78".to_string());
        assert_eq!(LispExp::Number(-36f64), eval_str(&exp_1).unwrap());
        assert_eq!(LispExp::Number(-78f64), eval_str(&exp_2).unwrap());
    }

    #[test]
    fn test_eval_fake_bool_str() {
        let exp_1 = LispExp::Str("true".to_string());
        let exp_1_p = LispExp::Str(" true ".to_string());
        let exp_2 = LispExp::Str("false".to_string());
        let exp_2_p = LispExp::Str("false ".to_string());
        assert_eq!(LispExp::Str("true".to_string()), eval_str(&exp_1).unwrap());
        assert_eq!(LispExp::Str("true".to_string()), eval_str(&exp_1_p).unwrap());
        assert_eq!(LispExp::Str("false".to_string()), eval_str(&exp_2).unwrap());
        assert_eq!(LispExp::Str("false".to_string()), eval_str(&exp_2_p).unwrap());
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
        LispExp::Str(x) => {
            match x.trim() {
                "true" | "false" => return Ok(LispExp::Str(x.trim().to_string())),
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
