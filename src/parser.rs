use crate::{LispErr, LispExp};

// #[test]
// fn test_parse_symbol() {
//     let tokens = vec!["+".to_string()];
//     let (exp, _) = parse(&tokens).unwrap();
//     assert_eq!(exp, LispExp::Symbol("+".into()));
// }

#[allow(dead_code)]
pub fn parse(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    let (token, rest) = tokens.split_first().unwrap();
    Ok((parse_atom(token), rest))
}

/// parse the token(num or symbol)
#[allow(dead_code)]
pub fn parse_atom(token: &str) -> LispExp {
    if let Ok(num) = token.parse::<f64>() {
        LispExp::Number(num)
    } else {
        LispExp::Symbol(token.into())
    }
}
