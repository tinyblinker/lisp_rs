use crate::{LispErr, LispExp};

/// parse() is the "Parser"'s main
#[allow(dead_code)]
pub fn parse(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    let (token, rest_tokens) = tokens
        .split_first()
        .ok_or(LispErr::Reason("parse(): no tokens to parse".to_string()))?;

    // as_str(): String->&str(borrow)
    match token.as_str() {
        "(" => read_seq(rest_tokens), // left parentheses->start to read the list
        ")" => Err(LispErr::Reason("Extra \')\'".to_string())),
        _ => Ok((parse_atom(token), rest_tokens)),
    }
}

/// Read a list: start after the left parenthesis, end when encountering )
fn read_seq(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    let mut elements = Vec::<LispExp>::new(); // empty list
    let mut remaining = tokens; // remaining tokens

    // keep looping until encounter ")"
    loop {
        let (token, rest_tokens) = remaining
            .split_first()
            .ok_or(LispErr::Reason("Missing \')\'".to_string()))?;
        if token == ")"{
            // encounter ")" -> list ends, return all collected elements
            return Ok((LispExp::List(elements),rest_tokens))
        }

        // Recursive: call parse to parse the next element
        // (this element itself might be a list)
        let (exp, new_rest_tokens) = parse(remaining)?;
        elements.push(exp);  // add to list
        remaining = new_rest_tokens;  // update the remaining tokens
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
