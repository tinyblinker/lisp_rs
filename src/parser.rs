use crate::{interner::intern, LispErr, LispExp};

/// parse() is the "Parser"'s main
#[allow(dead_code)]
pub fn parse<'a>(tokens: &'a [&'a str]) -> Result<(LispExp, &'a [&'a str]), LispErr> {
    // get just the first token
    let (token, rest_tokens) = tokens
        .split_first()
        .ok_or(LispErr::Reason("parse():No token left".into()))?;

    // match the first "token"
    match *token {
        "(" => read_seq(rest_tokens),
        ")" => Err(LispErr::Reason("Extra \')\'".into())),
        "'" => {
            let (quoted, the_rest) = parse(rest_tokens)?;
            Ok((
                LispExp::List(vec![LispExp::Symbol(intern("quote")), quoted]),
                the_rest,
            ))
        }
        _ => Ok((parse_atom(token), rest_tokens)),
    }
}

/// Read a list: start after the left parenthesis, end when encountering )
fn read_seq<'a>(tokens: &'a [&'a str]) -> Result<(LispExp, &'a [&'a str]), LispErr> {
    // create a Vector for holding the list_elements
    let mut list_elements: Vec<LispExp> = Vec::new();
    let mut tokens_left = tokens;
    loop {
        let (token, rest_tokens_a) = tokens_left
            .split_first()
            .ok_or(LispErr::Reason("Missing \')\'".to_string()))?;
        // if encounter the ')' then return the list and the rest_tokens immediately
        if *token == ")" {
            return Ok((LispExp::List(list_elements), rest_tokens_a));
        }
        let (exp, rest_tokens_b) = parse(tokens_left)?;
        list_elements.push(exp);
        tokens_left = rest_tokens_b;
    }
}

fn is_valid_string(s: &str) -> Result<(), LispErr> {
    let mut chars = s.chars();
    if chars.next() != Some('"') {
        return Err(LispErr::Reason(
            "parse_atom(): the String token must start with \'\"\'".to_string(),
        ));
    }
    // check 'esc' and 'closed' status
    let (esc, closed) = chars.try_fold((false, false), |(esc, _closed), ch| match (esc, ch) {
        (true, _) => Ok((false, false)),
        (_, '\\') => Ok((true, false)),
        (_, '"') => Ok((false, true)),
        (_, '\'') => Err(LispErr::Reason(
            "parse_atom(): the String token must use \'\\\' to esc".to_string(),
        )),
        _ => Ok((false, false)),
    })?;

    if esc || !closed {
        return Err(LispErr::Reason(
            "parse_atom(): the String token has a syntax error".to_string(),
        ));
    }
    Ok(())
}

/// parse_atom() is a part of the "Parser"
/// parse the atom(num or symbol)
pub fn parse_atom(token: &str) -> LispExp {
    if token == "#t" {
        LispExp::Bool(true)
    } else if token == "#f" {
        LispExp::Bool(false)
    } else if token == "nil" {
        LispExp::Nil
    } else if let Ok(num) = token.parse::<f64>() {
        LispExp::Number(num)
    } else {
        // we need to classify String & Symbol here
        if is_valid_string(token.into()).is_ok() {
            LispExp::String(token[1..(token.len() - 1)].into())
        } else {
            LispExp::Symbol(intern(token))
        }
    }
}
