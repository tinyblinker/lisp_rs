use crate::{LispErr, LispExp};

pub fn parse(tokens: &[String]) -> Result<(LispExp, &[String]), LispErr> {
    let (token, rest) = tokens
        .split_first()
        .ok_or(LispErr::Reason("No tokens left".to_string()))?;
    Ok((parse_atom(token), rest))
}

fn parse_atom(token: &String) -> LispExp {
    unimplemented!();
}
