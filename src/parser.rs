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
    Ok((LispExp::Symbol(token.to_string()), rest))
}
