use crate::{interner::lookup, LispErr, LispExp};
/// Display trait can replace this one for better accessiblity
/// this function just left here for referrence
pub fn friendly_check_lispexp_list(a_lisp_list: LispExp) -> Result<String, LispErr> {
    let mut res_str = String::new();
    match a_lisp_list {
        LispExp::List(elements) => {
            res_str.push('(');
            for i in elements {
                match i {
                    LispExp::List(_) => {
                        res_str.push_str(friendly_check_lispexp_list(i.clone()).unwrap().as_str());
                    }
                    LispExp::Symbol(x) => {
                        res_str.push_str(lookup(x).as_str());
                        res_str.push(' ');
                    }
                    LispExp::Number(n) => {
                        res_str.push_str(format!("{}", n).as_str());
                        res_str.push(' ');
                    }
                    _ => res_str.push('?'),
                }
            }
            res_str.push(')');
            Ok(res_str)
        }
        _ => {
            return Err(LispErr::Reason(
                "\'friendly_check_lispexp_list()\':please input a \'LispExp::List\'".into(),
            ))
        }
    }
}
