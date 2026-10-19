use crate::intern;
use crate::interner::predefined;
use crate::number;
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
                    if *s == predefined().if_sym && elements.len() == 4 {
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
                    else if *s == predefined().define && elements.len() == 3 {
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
                    else if *s == predefined().lambda && elements.len() >= 3 {
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
                    } else if *s == predefined().begin {
                        let args = &elements[1..];
                        if args.is_empty() {
                            return Ok(LispExp::Nil);
                        }
                        // eval all but last(for side effects)
                        for arg in &args[..args.len() - 1] {
                            eval(arg, &mut current_env)?;
                        }
                        // last expression is in tail position=>TCO
                        current_exp = args.last().unwrap().clone();
                        continue;
                    } else if *s == predefined().set_bang {
                        let args = &elements[1..];
                        if args.len() != 2 {
                            *env = current_env;
                            return Err(LispErr::Reason("\'set!\' needs 2 arguments".into()));
                        }
                        if let LispExp::Symbol(name) = &args[0] {
                            let value = eval(&args[1], &mut current_env)?;
                            current_env.set_upward(*name, value)?;
                            *env = current_env;
                            return Ok(LispExp::Nil);
                        }
                    } else if *s == predefined().let_sym {
                        let args = &elements[1..];
                        // parse the binding list
                        let bindings = &args[0];
                        let body_exprs = &args[1..];
                        let mut names: Vec<LispExp> = Vec::new();
                        let mut vals: Vec<LispExp> = Vec::new();
                        if let LispExp::List(binds) = bindings {
                            for bind in binds {
                                if let LispExp::List(one_bind) = bind {
                                    if let LispExp::Symbol(one_name) = &one_bind[0] {
                                        names.push(LispExp::Symbol(*one_name));
                                        vals.push(one_bind[1].clone())
                                    }
                                }
                            }
                        }
                        // body wrapping(multi-expression -> implicit begin)
                        let body = if body_exprs.len() == 1 {
                            body_exprs[0].clone()
                        } else {
                            LispExp::List(
                                std::iter::once(LispExp::Symbol(predefined().begin))
                                    .chain(body_exprs.iter().cloned())
                                    .collect(),
                            )
                        };
                        // conver to a lambda
                        let lambda = LispExp::List(vec![
                            LispExp::Symbol(predefined().lambda),
                            LispExp::List(names),
                            body,
                        ]);
                        let mut call = vec![lambda];
                        call.extend(vals);
                        current_exp = LispExp::List(call);
                        continue;
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

pub fn create_default_env() -> LispEnv {
    let mut env = LispEnv::new();
    // set "+" function
    env.set(
        intern("+"),
        LispExp::Func(|args| {
            let sum: f64 = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(n) = arg {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .sum();
            Ok(number!(sum))
        }),
    );
    // set "-" function
    env.set(
        intern("-"),
        LispExp::Func(|args| {
            let nums: Vec<f64> = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(n) = arg {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            if nums.len() == 1 {
                Ok(number!(-nums[0]))
            } else {
                Ok(number!(nums[0] - nums[1..].iter().sum::<f64>()))
            }
        }),
    );
    // set "*" function
    env.set(
        intern("*"),
        LispExp::Func(|args| {
            let result: f64 = args.iter().fold(1.0f64, |acc, arg| match arg {
                LispExp::Number(num) => num * acc,
                _ => acc,
            });
            Ok(number!(result))
        }),
    );
    // set "/" function
    env.set(
        intern("/"),
        LispExp::Func(|args| {
            let nums: Vec<f64> = args
                .iter()
                .filter_map(|arg| {
                    if let LispExp::Number(num) = arg {
                        Some(*num)
                    } else {
                        None
                    }
                })
                .collect();
            if nums.len() == 1 {
                Ok(number!(1f64 / nums[0]))
            } else if nums[1..].contains(&0f64) {
                Err(LispErr::Reason(
                    "Cannot divide \'0\' in division".to_string(),
                ))
            } else {
                let result: f64 = nums[1..].iter().fold(nums[0], |acc, num| acc / num);
                Ok(number!(result))
            }
        }),
    );
    // square function
    env.set(
        intern("square"),
        LispExp::Func(|args| {
            if let LispExp::Number(num) = &args[0] {
                Ok(number!(num * num))
            } else {
                Err(LispErr::Reason("square() needs a number".to_string()))
            }
        }),
    );
    // "=" comparison
    env.set(
        intern("="),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x == y))
            } else {
                Err(LispErr::Reason("\'=\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // ">" comparison
    env.set(
        intern(">"),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x > y))
            } else {
                Err(LispErr::Reason("\'>\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // ">=" comparison
    env.set(
        intern(">="),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x >= y))
            } else {
                Err(LispErr::Reason(
                    "\'>=\' needs a pair of numbers".to_string(),
                ))
            }
        }),
    );
    // "<" comparison
    env.set(
        intern("<"),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x < y))
            } else {
                Err(LispErr::Reason("\'<\' needs a pair of numbers".to_string()))
            }
        }),
    );
    // "<=" comparison
    env.set(
        intern("<="),
        LispExp::Func(|args| {
            if let (LispExp::Number(x), LispExp::Number(y)) = (&args[0], &args[1]) {
                Ok(LispExp::Bool(x <= y))
            } else {
                Err(LispErr::Reason(
                    "\'<=\' needs a pair of numbers".to_string(),
                ))
            }
        }),
    );
    // "string-length" function
    env.set(
        intern("string-length"),
        LispExp::Func(|args| {
            if let LispExp::String(str) = &args[0] {
                Ok(LispExp::Number(str.len() as f64))
            } else {
                Err(LispErr::Reason(
                    "\'string-length function\' needs a \'String\'".to_string(),
                ))
            }
        }),
    );
    env
}
