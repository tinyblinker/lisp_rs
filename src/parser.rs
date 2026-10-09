use crate::{LispErr, LispExp};

/// parse() is the "Parser"'s main
#[allow(dead_code)]
pub fn parse(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    // get just the first token
    let (token, rest_tokens) = tokens
        .split_first()
        .ok_or(LispErr::Reason("parse():No token left".into()))?;

    // match the first "token"
    match token.as_str() {
        "(" => read_seq(rest_tokens),
        ")" => Err(LispErr::Reason("Extra \')\'".into())),
        _ => Ok((parse_atom(token), rest_tokens)),
    }
}

/// Read a list: start after the left parenthesis, end when encountering )
fn read_seq(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    // create a Vector for holding the list_elements
    let mut list_elements: Vec<LispExp> = Vec::new();
    let mut tokens_left = tokens;

    loop {
        let (token, rest_tokens_a) = tokens_left
            .split_first()
            .ok_or(LispErr::Reason("Missing \')\'".to_string()))?;
        // if encounter the ')' then return the list and the rest_tokens immediately
        if token == ")" {
            return Ok((LispExp::List(list_elements), rest_tokens_a));
        }
        let (exp, rest_tokens_b) = parse(tokens_left)?;
        list_elements.push(exp);
        tokens_left = rest_tokens_b;
    }
}

/// parse_atom() is a part of the "Parser"
/// parse the atom(num or symbol)
fn parse_atom(token: &str) -> LispExp {
    if let Ok(num) = token.parse::<f64>() {
        LispExp::Number(num)
    } else {
        LispExp::Symbol(token.into())
    }
}
