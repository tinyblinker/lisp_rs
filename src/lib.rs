#[derive(Clone, Debug, PartialEq)]
pub enum LispExp{
    Number(f64), // number type
}

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr{
    Reason(String), // error message
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_create_number(){
        let n = LispExp::Number(88.0);
        assert_eq!(n, LispExp::Number(88.0));
    }
}
