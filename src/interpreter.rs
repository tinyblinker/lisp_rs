use crate::intern;
use crate::LispLambda;
use crate::RefCell;
use crate::{env::LispEnv, LispErr, LispExp};
use std::rc::Rc;

/// eval() is the "Evaluator"
#[allow(dead_code)]
pub fn eval(exp: &LispExp, env: &mut LispEnv) -> Result<LispExp, LispErr> {
    // 01 Initialize: clone the current expresstion, take ownership of env
    let mut current_exp = exp.clone();
    let mut current_env = std::mem::take(env);
    // 02 trampoline loop: each loop processes one expression
    loop {
        match &current_exp {
            // -- Self-evaluating type: return directly --
            LispExp::Number(_)
            | LispExp::Bool(_)
            | LispExp::Nil
            | LispExp::Func(_)
            | LispExp::String(_)
            | LispExp::Lambda(_) => {
                *env = current_env; // return the original environment
                return Ok(current_exp.clone());
            }
            // -- Symbols: look up environment --
            LispExp::Symbol(x) => {
                let res = current_env.get(*x);
                *env = current_env; // return the original environment
                return res;
            }
            // -- Lists --
            LispExp::List(elements) => {
                if elements.is_empty() {
                    *env = current_env; // return the original environment
                    return Ok(LispExp::Nil);
                }
                // -- "if" (tail position optimization) --
                if let LispExp::Symbol(s) = &elements[0] {
                    // (if condition true_branch false_branch)
                    if *s == intern("if") && elements.len() == 4 {
                        let cond = eval(&elements[1], &mut current_env)?;
                        let is_true = !matches!(cond, LispExp::Bool(false) | LispExp::Nil);
                        // TCO!!!! do not recursively eval, update expression then continue
                        current_exp = if is_true {
                            elements[2].clone()
                        } else {
                            elements[3].clone()
                        };
                        continue;
                    }
                    // -- "define" (supports  recursive definition) --
                    else if *s == intern("define") && elements.len() == 3 {
                        println!("###[debug]elements = {:?};", elements);
                        println!("###[debug]elements[1] = {:?};", elements[1]);
                        // use "Rc" to share environment: let closures "sees" themselves
                        if let LispExp::Symbol(name) = &elements[1] {
                            let share_env = Rc::new(RefCell::new(std::mem::take(&mut current_env)));
                            // first place a placeholder (will be replace with real value)
                            share_env.borrow_mut().set(name.clone(), LispExp::Nil);
                            // current_env takes shared_env as outer
                            current_env = LispEnv::with_outer(share_env.clone());
                            // evaluate (lambda's env shares the same "Rc" through outer)
                            let value = eval(&elements[2], &mut current_env)?;
                            // [KEY!!!] eval's internal TCO may have modified current_env
                            // must rebuild current_env pointer to shared_env
                            current_env = LispEnv::with_outer(share_env.clone());
                            // replace placeholder with real value
                            share_env.borrow_mut().set(name.clone(), value);
                            *env = current_env; // return the original environment
                            return Ok(LispExp::Nil);
                        } else {
                            *env = current_env; // return the original environment
                            return Err(LispErr::Reason(
                                "eval(): define needs a \'Symbol\'".to_string(),
                            ));
                        }
                    }
                    // -- lambda --
                    else if *s == intern("lambda") && elements.len() >= 3 {
                        let params: Vec<u64> = match &elements[1] {
                            LispExp::List(param_list) => param_list
                                .iter()
                                .map(|param| {
                                    if let LispExp::Symbol(x) = param {
                                        x.clone()
                                    } else {
                                        intern("?") // return "?" if param_list's element is not a symbol
                                    }
                                })
                                .collect(),
                            _ => {
                                *env = current_env;
                                return Err(LispErr::Reason(
                                    "eval(): lambda\'s parament must be a list".to_string(),
                                ));
                            }
                        };
                        let body = elements[2].clone();
                        let lambda = LispExp::Lambda(Box::new(LispLambda {
                            params: params,
                            body: Box::new(body),
                            // Lambda.env points to the definition-time CallFrame
                            env: Rc::new(RefCell::new(current_env.clone())),
                        }));
                        *env = current_env;
                        return Ok(lambda);
                    }
                }
                // -- Ordinary function call --
                let func = eval(&elements[0], &mut current_env)?;
                let args: Result<Vec<LispExp>, LispErr> = elements[1..]
                    .iter()
                    .map(|arg| eval(arg, &mut current_env))
                    .collect();
                match func {
                    LispExp::Func(f) => {
                        *env = current_env;
                        return f(&(args?));
                    }
                    LispExp::Lambda(lambda) => {
                        let mut new_env: LispEnv = LispEnv::with_outer(lambda.env.clone());
                        // bind parameter names to argument values one by one(set them in cloned environment)
                        for (param, arg) in lambda.params.iter().zip(args?.iter()) {
                            new_env.set(param.clone(), arg.clone());
                        }
                        // TCO! Update expression and environment, continue
                        current_exp = lambda.body.as_ref().clone();
                        current_env = new_env;
                        continue;
                    }
                    _ => {
                        *env = current_env;
                        return Err(LispErr::Reason("Not a function".to_string()));
                    }
                }
            }
        }
    }
}
