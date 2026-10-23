use crate::debug_helper::friendly_check_lispexp_list;
use crate::intern;
use crate::interner::lookup;
use crate::interner::predefined;
use crate::interner::PredefinedSyms;
use crate::number;
use crate::LispLambda;
use crate::RefCell;
use crate::{env::LispEnv, LispErr, LispExp};
use std::io::BufRead;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

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
            | LispExp::Macro(_)
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
                        let (my_params, my_rest_params) = match &elements[1] {
                            LispExp::List(my_lambda_args) => {
                                eval_lambda_get_paras_helper(my_lambda_args).unwrap()
                            }
                            _ => {
                                return Err(LispErr::Reason(
                                    "\'lambda\' 's parameter must be a list".into(),
                                ))
                            }
                        };
                        let body = elements[2].clone();
                        let lambda = LispExp::Lambda(Box::new(LispLambda {
                            params: my_params,
                            body: Box::new(body),
                            // Lambda.env points to the definition-time CallFrame
                            env: Rc::new(RefCell::new(current_env.clone())),
                            rest: my_rest_params,
                        }));
                        *env = current_env;
                        return Ok(lambda);
                    }
                    // -- defmacro --
                    else if *s == predefined().defmacro_sym {
                        let (my_params, my_rest_params) = match &elements[2] {
                            LispExp::List(my_defmacro_lambda_args) => {
                                eval_defmacro_get_lambda_paras_helper(my_defmacro_lambda_args)
                                    .unwrap()
                            }
                            _ => {
                                return Err(LispErr::Reason(
                                    "\'lambda\' in \'defmacro\' 's parameter must be a list".into(),
                                ));
                            }
                        };
                        let body = elements[3].clone();
                        let my_macro = LispExp::Macro(Box::new(LispLambda {
                            params: my_params,
                            body: Box::new(body),
                            // Lambda.env points to the definition-time CallFrame
                            env: Rc::new(RefCell::new(current_env.clone())),
                            rest: my_rest_params,
                        }));
                        let macro_name = {
                            match &elements[1] {
                                LispExp::Symbol(n) => n.clone(),
                                _ => {
                                    return Err(LispErr::Reason("\'defmacro\' 's macro name should be a \'LispExp::Symbol\'".into()));
                                }
                            }
                        };
                        current_env.set(macro_name, my_macro);
                        *env = current_env;
                        return Ok(LispExp::Nil);
                    }
                    // -- begin --
                    else if *s == predefined().begin {
                        let args = &elements[1..];
                        if args.is_empty() {
                            *env = current_env;
                            return Ok(LispExp::Nil);
                        }
                        // eval all but last(for side effects)
                        for arg in &args[..args.len() - 1] {
                            eval(arg, &mut current_env)?;
                        }
                        // last expression is in tail position=>TCO
                        current_exp = args.last().unwrap().clone();
                        continue;
                    }
                    // -- set! --
                    else if *s == predefined().set_bang {
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
                    }
                    // -- let --
                    else if *s == predefined().let_sym {
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
                        // convert to a lambda
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
                    // -- cond --
                    else if *s == predefined().cond_sym {
                        let args = &elements[1..];
                        let mut new_expr: Option<LispExp> = None;
                        for branches in args {
                            if let LispExp::List(branch) = branches {
                                if branch.is_empty() {
                                    continue;
                                }
                                let conditon = &branch[0];
                                let body = &branch[1..];
                                let is_else = matches!(conditon, LispExp::Symbol(symbol_id) if lookup(*symbol_id) == "else");
                                let passed = is_else || {
                                    let val = eval(conditon, &mut current_env)?;
                                    !matches!(val, LispExp::Nil | LispExp::Bool(false))
                                };
                                if passed {
                                    if body.is_empty() {
                                        *env = current_env;
                                        return Ok(LispExp::Nil);
                                    }
                                    // TCO: execute body
                                    new_expr = Some(if body.len() == 1 {
                                        body[0].clone()
                                    } else {
                                        LispExp::List(
                                            std::iter::once(LispExp::Symbol(predefined().begin))
                                                .chain(body.iter().cloned())
                                                .collect(),
                                        )
                                    });
                                    break;
                                }
                            }
                        }
                        if let Some(expr) = new_expr {
                            current_exp = expr;
                            continue;
                        }
                        *env = current_env;
                        return Ok(LispExp::Nil);
                    }
                    // -- and --
                    else if *s == predefined().and_sym {
                        let args = &elements[1..];
                        if args.is_empty() {
                            *env = current_env;
                            return Ok(LispExp::Bool(true));
                        }
                        for args in &args[..args.len() - 1] {
                            let val = eval(args, &mut current_env)?;
                            if matches!(val, LispExp::Bool(false) | LispExp::Nil) {
                                *env = current_env;
                                return Ok(val);
                            }
                        }
                        current_exp = args.last().unwrap().clone(); // TCO for last
                        continue;
                    }
                    // -- or --
                    else if *s == predefined().or_sym {
                        let args = &elements[1..];
                        if args.is_empty() {
                            *env = current_env;
                            return Ok(LispExp::Bool(false));
                        }
                        for arg in &args[..args.len() - 1] {
                            let val = eval(arg, &mut current_env)?;
                            if !matches!(val, LispExp::Nil | LispExp::Bool(false)) {
                                *env = current_env;
                                return Ok(val);
                            }
                        }
                        // TCO
                        current_exp = args.last().unwrap().clone();
                        continue;
                    }
                    // -- let* --
                    else if *s == predefined().let_star {
                        let args = &elements[1..];
                        let bindings = &args[0];
                        let body_exprs = &args[1..];
                        // parse the binding lists
                        let binds: Vec<&LispExp> = if let LispExp::List(the_binds) = bindings {
                            the_binds.iter().collect()
                        } else {
                            vec![]
                        };
                        // parse the body to eval
                        let body = if body_exprs.len() == 1 {
                            body_exprs[0].clone()
                        } else {
                            LispExp::List(
                                std::iter::once(LispExp::Symbol(predefined().begin))
                                    .chain(body_exprs.iter().cloned())
                                    .collect(),
                            )
                        };
                        // make embebbed 'let'
                        let mut result = body.clone();
                        for bind in binds.iter().rev() {
                            if let LispExp::List(the_bind) = bind {
                                if the_bind.len() >= 2 {
                                    if let LispExp::Symbol(symbol_name) = &the_bind[0] {
                                        let val = the_bind[1].clone();
                                        result = LispExp::List(vec![
                                            LispExp::Symbol(predefined().let_sym),
                                            LispExp::List(vec![LispExp::List(vec![
                                                LispExp::Symbol(*symbol_name),
                                                val,
                                            ])]),
                                            result,
                                        ]);
                                    }
                                }
                            }
                        }
                        if binds.is_empty() {
                            result = body;
                        }
                        current_exp = result;
                        continue; // TCO!
                    }
                    // -- quote --
                    else if *s == predefined().quote_sym {
                        let args = &elements[1..];
                        *env = current_env;
                        return Ok(args[0].clone());
                    }
                    // -- letrec --
                    else if *s == predefined().let_rec {
                        let args = &elements[1..];
                        let bindings = &args[0];
                        let body_expr = &args[1..];
                        // create shared environment(clone current env to preserve builtins)
                        let shared_env = Rc::new(RefCell::new(current_env.clone()));
                        // write all names as placeholder(nil)(when use before bind, return nil as error)
                        if let LispExp::List(all_binds_lists) = bindings {
                            for the_bind in all_binds_lists {
                                if let LispExp::List(b) = the_bind {
                                    if let LispExp::Symbol(n) = &b[0] {
                                        shared_env.borrow_mut().set(*n, LispExp::Nil);
                                    }
                                }
                            }
                        }
                        // evaluate lambdas in env that can see the whiteboard,then replace placeholder
                        let mut eval_env = LispEnv::with_outer(shared_env.clone());
                        if let LispExp::List(all_binds_list) = bindings {
                            for the_bind in all_binds_list {
                                if let LispExp::List(b) = the_bind {
                                    if b.len() >= 2 {
                                        if let LispExp::Symbol(n) = &b[0] {
                                            let val = eval(&b[1], &mut eval_env)?;
                                            shared_env.borrow_mut().set(*n, val);
                                        }
                                    }
                                }
                            }
                        }
                        // evaluate the body in share env(TCO)
                        let body = if body_expr.len() == 1 {
                            body_expr[0].clone()
                        } else {
                            LispExp::List(
                                std::iter::once(LispExp::Symbol(predefined().begin))
                                    .chain(body_expr.iter().cloned())
                                    .collect(),
                            )
                        };
                        current_env = LispEnv::with_outer(shared_env);
                        current_exp = body;
                        continue;
                    } else if *s == predefined().quasiquote {
                        let args = &elements[1..];
                        let expanded = quasiquote_expand_helper(&args[0], &predefined()).unwrap();
                        // TCO!
                        current_exp = expanded;
                        continue;
                    }
                    // -- Macro Unfold --
                    if let Ok(LispExp::Macro(my_macro)) = current_env.get(*s) {
                        let args = &elements[1..];
                        let mut new_env: LispEnv = LispEnv::with_outer(my_macro.env.clone());
                        // bind parameter names to its' vals one by one in the cloned env
                        for (param, arg) in my_macro.params.iter().zip(args.iter()) {
                            new_env.set(param.clone(), arg.clone());
                        }
                        // bind the "rest" for variabic function
                        if let Some(rest_id) = my_macro.rest {
                            let extra = args[my_macro.params.len()..].to_vec();
                            new_env.set(rest_id, LispExp::List(extra));
                        }
                        // TCO!!!
                        // macro "expanded" from "RAW" AST to evaluable "current_exp"
                        // -->call the "MACRO" to get the expanded code
                        println!("####[debug]current_exp = {:?};", current_exp);
                        current_exp = eval(&my_macro.body, &mut new_env)?;
                        println!("move here 01");
                        println!(
                            "####[debug][in the 01]current_exp = {:?};",
                            friendly_check_lispexp_list(current_exp.clone()).unwrap()
                        );
                        continue;
                    }
                }
                // -- Ordinary function call --
                println!("move here 02");
                println!(
                    "####[debug][in the 02]elements[0] = {:?};",
                    friendly_check_lispexp_list(LispExp::List(vec![elements[0].clone()])).unwrap()
                );
                let func = eval(&elements[0], &mut current_env)?;
                // this unfold the args' AST, macro should not unfold args' AST,so "macro unfold"
                // should not be here
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
                        for (param, arg) in lambda.params.iter().zip(args.clone()?.iter()) {
                            new_env.set(param.clone(), arg.clone());
                        }
                        // bind the "rest" for variabic function
                        if let Some(rest_id) = lambda.rest {
                            let extra = (args.clone()?)[lambda.params.len()..].to_vec();
                            new_env.set(rest_id, LispExp::List(extra));
                        }
                        // TCO! Update expression and environment, continue
                        current_exp = lambda.body.as_ref().clone();
                        current_env = new_env;
                        continue;
                    }
                    not_a_function => {
                        *env = current_env;
                        match not_a_function {
                            LispExp::Symbol(s) => {
                                return Err(LispErr::Reason(format!(
                                    "{:?} isn't a function, we eval the 1st element of a list as func, but got a \'LispExp::Symbol\'",
                                    lookup(s)
                                )));
                            }
                            _ => {
                                return Err(LispErr::Reason(format!(
                                    "\'{:?}\' isn't a function",
                                    not_a_function
                                )));
                            }
                        }
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
    // "not" function
    env.set(
        intern("not"),
        LispExp::Func(|args| match &args[0] {
            LispExp::Nil | LispExp::Bool(false) => Ok(LispExp::Bool(true)),
            _ => Ok(LispExp::Bool(false)),
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
    // "list" function
    env.set(
        intern("list"),
        LispExp::Func(|args| Ok(LispExp::List(args.to_vec()))),
    );
    // "cons" function
    env.set(
        intern("cons"),
        LispExp::Func(|args| match &args[1] {
            LispExp::List(els) => {
                let mut new_list = vec![args[0].clone()];
                new_list.extend(els.clone());
                Ok(LispExp::List(new_list))
            }
            LispExp::Nil => Ok(LispExp::List(vec![args[0].clone()])),
            _ => Err(LispErr::Reason(
                "\'cons\' second argument must be a list".into(),
            )),
        }),
    );
    // "car" function
    env.set(
        intern("car"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'car\' have no arg".into()));
            };
            match &args[0] {
                LispExp::List(list_content) => {
                    if list_content.is_empty() {
                        return Err(LispErr::Reason(
                            "\'car\' cannot receives empty \'LispExp::List\' as the arg".into(),
                        ));
                    }
                    Ok(list_content[0].clone())
                }
                LispExp::Nil => Err(LispErr::Reason(
                    "\'car\' only receives \'LispExp::List\' as the arg".into(),
                )),
                _ => Err(LispErr::Reason(
                    "\'car\' only receives \'LispExp::List\' as the arg".into(),
                )),
            }
        }),
    );
    // "cdr" function
    env.set(
        intern("cdr"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'cdr\' have no arg".into()));
            };
            match &args[0] {
                LispExp::List(list_content) => {
                    if list_content.is_empty() {
                        return Err(LispErr::Reason(
                            "\'cdr\' cannot receives empty \'LispExp::List\' as the arg".into(),
                        ));
                    }
                    Ok(LispExp::List(list_content[1..].to_vec()))
                }
                LispExp::Nil => Err(LispErr::Reason(
                    "\'cdr\' only receives \'LispExp::List\' as the arg".into(),
                )),
                _ => Err(LispErr::Reason(
                    "\'cdr\' only receives \'LispExp::List\' as the arg".into(),
                )),
            }
        }),
    );
    // "cadr" function
    env.set(
        intern("cadr"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'cadr\' have no arg".into()));
            };
            match &args[0] {
                LispExp::List(list_content) => {
                    if list_content.len() == 0 {
                        return Err(LispErr::Reason(
                            "\'cadr\' cannot receives empty \'LispExp::List\' as the arg".into(),
                        ));
                    } else if list_content.len() == 1 {
                        return Err(LispErr::Reason(
                            "\'cadr\' find the \'LispExp::List\' is too short".into(),
                        ));
                    }
                    Ok(list_content[1].clone())
                }
                LispExp::Nil => Err(LispErr::Reason(
                    "\'cadr\' only receives \'LispExp::List\' as the arg".into(),
                )),
                _ => Err(LispErr::Reason(
                    "\'cadr\' only receives \'LispExp::List\' as the arg".into(),
                )),
            }
        }),
    );
    // "caddr" function
    env.set(
        intern("caddr"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'caddr\' have no arg".into()));
            };
            match &args[0] {
                LispExp::List(list_content) => {
                    if list_content.len() == 0 {
                        return Err(LispErr::Reason(
                            "\'caddr\' cannot receives empty \'LispExp::List\' as the arg".into(),
                        ));
                    } else if list_content.len() <= 2 {
                        return Err(LispErr::Reason(
                            "\'caddr\' find the \'LispExp::List\' is too short".into(),
                        ));
                    }
                    Ok(list_content[2].clone())
                }
                LispExp::Nil => Err(LispErr::Reason(
                    "\'caddr\' only receives \'LispExp::List\' as the arg".into(),
                )),
                _ => Err(LispErr::Reason(
                    "\'caddr\' only receives \'LispExp::List\' as the arg".into(),
                )),
            }
        }),
    );
    // "append" function
    env.set(
        intern("append"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'append\' have no arg".into()));
            }
            let mut new_vec: Vec<LispExp> = Vec::new();
            let _ = args.iter().try_for_each(|x| {
                Ok(match x {
                    LispExp::List(x) => {
                        for exp in x {
                            new_vec.push(exp.clone());
                        }
                    }
                    LispExp::Nil => (),
                    _ => {
                        return Err(LispErr::Reason(
                            "\'append\' only receive \'LispExp::List\' as arg(s)".into(),
                        ));
                    }
                })
            });
            Ok(LispExp::List(new_vec))
        }),
    );
    // "length" function
    env.set(
        intern("length"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'length\' have no args".into()));
            };
            match &args[0] {
                LispExp::List(x) => Ok(LispExp::Number(x.len() as f64)),
                LispExp::Nil => Ok(LispExp::Number(0f64)),
                _ => Err(LispErr::Reason(
                    "\'length\' only receive \'LispExp::List\'".into(),
                )),
            }
        }),
    );
    // "reverse" function
    env.set(
        intern("reverse"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'reverse\' have no args".into()));
            };
            match &args[0] {
                LispExp::List(x) => {
                    let mut y = x.clone();
                    y.reverse();
                    Ok(LispExp::List(y))
                }
                LispExp::Nil => Ok(LispExp::Nil),
                _ => todo!(),
            }
        }),
    );
    // "member" function
    env.set(
        intern("member"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'member\' have no args".into()));
            };
            match &args[1] {
                LispExp::List(contents) => {
                    for (i, c) in contents.iter().enumerate() {
                        if c == &args[0] {
                            return Ok(LispExp::List(contents[i..].to_vec()));
                        }
                    }
                    return Ok(LispExp::Bool(false));
                }
                _ => {
                    return Err(LispErr::Reason(
                        "\'member\' only receive \'LispExp::List\'".into(),
                    ));
                }
            }
        }),
    );
    // "null?" function
    env.set(
        intern("null?"),
        LispExp::Func(|args| Ok(LispExp::Bool(matches!(args[0], LispExp::Nil)))),
    );
    // "map" function
    env.set(
        intern("map"),
        LispExp::Func(|args| {
            // take args[0] as function
            let func = &args[0];
            let list = match &args[1] {
                LispExp::List(els) => els,
                LispExp::Nil => {
                    return Ok(LispExp::Nil);
                }
                _ => {
                    return Err(LispErr::Reason(
                        "\'map\' second argument must be a list".into(),
                    ));
                }
            };
            let mut res = Vec::new();
            for el in list {
                match func {
                    LispExp::Func(f) => res.push(f(&[el.clone()])?),
                    LispExp::Lambda(lam) => {
                        let mut env = LispEnv::with_outer(lam.env.clone());
                        if let Some(p) = lam.params.first() {
                            env.set(*p, el.clone());
                        }
                        res.push(eval(&lam.body, &mut env)?);
                    }
                    _ => {
                        return Err(LispErr::Reason(
                            "\'map\' first argument must be a function".into(),
                        ))
                    }
                }
            }
            Ok(LispExp::List(res))
        }),
    );
    // "number?" function
    env.set(
        intern("number?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(args[0], LispExp::Number(_))));
        }),
    );
    // "symbol?" function
    env.set(
        intern("symbol?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(args[0], LispExp::Symbol(_))));
        }),
    );
    // "boolean?" function
    env.set(
        intern("boolean?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(args[0], LispExp::Bool(_))));
        }),
    );
    // "string?" function
    env.set(
        intern("string?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(args[0], LispExp::String(_))));
        }),
    );
    // "string?" function
    env.set(
        intern("procedure?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(
                args[0],
                LispExp::Func(_) | LispExp::Lambda(_)
            )));
        }),
    );
    // "pair?" function
    env.set(
        intern("pair?"),
        LispExp::Func(|args| {
            Ok(LispExp::Bool(
                matches!(&args[0], LispExp::List(els) if !els.is_empty()),
            ))
        }),
    );
    // "list?" function
    env.set(
        intern("list?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(matches!(
                &args[0],
                LispExp::List(_) | LispExp::Nil
            )));
        }),
    );
    // "eq?" function
    env.set(
        intern("eq?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(&args[0] == &args[1]));
        }),
    );
    // "equal?" function
    env.set(
        intern("equal?"),
        LispExp::Func(|args| {
            return Ok(LispExp::Bool(equal_function_helper(&args[0], &args[1])));
        }),
    );
    // "apply" function
    env.set(
        intern("apply"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'apply\' have no args".into()));
            } else if args.len() == 1 {
                return Err(LispErr::Reason("\'apply\' should have two args".into()));
            } else if args.len() == 2 {
                let func = args[0].clone();
                let f_args: Vec<LispExp> = match &args[1] {
                    LispExp::List(the_args) => {
                        the_args.clone()
                    }
                    LispExp::Nil=>{
                        return Ok(LispExp::Nil);
                    },
                    _ => {
                        return Err(LispErr::Reason(
                            "\'apply\' 's 2nd arg should be a \'LispExp::List\'".into(),
                        ));
                    }
                };
                match func {
                    LispExp::Func(f) => {
                        return f(&(f_args));
                    }
                    LispExp::Lambda(lambda) => {
                        let mut new_env:LispEnv = LispEnv::with_outer(lambda.env.clone());
                        // bind parameter names to argument values one by one(in cloned environment)
                        for (param, arg) in lambda.params.iter().zip(f_args.iter()) {
                            new_env.set(param.clone(), arg.clone());
                        }
                        let res = eval(&lambda.body,&mut new_env)?;
                        return Ok(res)
                    },
                    _ => {
                        return Err(LispErr::Reason(
                            "\'apply\' 's 1st arg should be a \'LispExp::Lambda\' or \'LispExp::Func\'".into(),
                        ));
                    }
                }
            } else {
                return Err(LispErr::Reason("\'apply\' should only have two args".into()));
            }
        }),
    );
    // "filter" function
    env.set(
        intern("filter"),
        LispExp::Func(|args| {
            if args.is_empty() {
                return Err(LispErr::Reason("\'filter\' have no args".into()));
            } else if args.len() == 1 {
                return Err(LispErr::Reason("\'filter\' should have two args".into()));
            } else if args.len() == 2 {
                let func = args[0].clone();
                let f_args: Vec<LispExp> = match &args[1] {
                    LispExp::List(the_args) => the_args.clone(),
                    LispExp::Nil => {
                        return Ok(LispExp::Nil);
                    }
                    _ => {
                        return Err(LispErr::Reason(
                            "\'filter\' 's 2nd arg should be a \'LispExp::List\'".into(),
                        ));
                    }
                };
                let mut new_list: Vec<LispExp> = Vec::new();
                for the_arg in f_args {
                    match &func {
                        LispExp::Lambda(lambda) => {
                            // bind parameter names to argument values one by one(in cloned environment)
                            let mut new_env: LispEnv = LispEnv::with_outer(lambda.env.clone());
                            new_env.set(lambda.params[0], the_arg.clone());
                            let abc = eval(&lambda.body, &mut new_env)?;
                            if abc == LispExp::Bool(true) {
                                new_list.push(the_arg.clone());
                            }
                        }
                        _ => {
                            return Err(LispErr::Reason(
                                "\'filter\' 's 1st arg should be a \'LispExp::Lambda\'".into(),
                            ));
                        }
                    }
                }
                return Ok(LispExp::List(new_list));
            } else {
                return Err(LispErr::Reason(
                    "\'filter\' should only have two args".into(),
                ));
            }
        }),
    );
    // -- gensym --
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    env.set(
        intern("gensym"),
        LispExp::Func(|args| {
            let perfix = if let Some(LispExp::String(s)) = args.first() {
                s.clone()
            } else {
                "g".into()
            };
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            Ok(LispExp::Symbol(intern(&format!("{}_{}", perfix, id))))
        }),
    );
    // -- error --
    env.set(
        intern("error"),
        LispExp::Func(|args| {
            let msg = args
                .first()
                .map(|a| format!("{}", a))
                .unwrap_or("error".into());
            Err(LispErr::Reason(msg))
        }),
    );
    // -- display --
    env.set(
        intern("display"),
        LispExp::Func(|args| {
            if let Some(arg) = args.first() {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                display_core_helper(&mut out, arg)
                    .map_err(|err| LispErr::Reason(format!("\'display\': {err}")))?;
            }
            Ok(LispExp::Nil)
        }),
    );
    // -- newline -- print a line break
    env.set(
        intern("newline"),
        LispExp::Func(|_args| {
            println!();
            Ok(LispExp::Nil)
        }),
    );
    // -- read -- read user inputs
    env.set(
        intern("read"),
        LispExp::Func(|_args| {
            let stdin = std::io::stdin();
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => Ok(LispExp::Nil),
                Ok(_) => Ok(LispExp::String(line.trim_end().to_string())),
                Err(_) => Err(LispErr::Reason("read: failed to read input".into())),
            }
        }),
    );
    env
}

pub fn display_core_helper<W: Write>(out: &mut W, arg: &LispExp) -> std::io::Result<()> {
    write!(out, "{}", arg)
}

fn equal_function_helper(expr_a: &LispExp, expr_b: &LispExp) -> bool {
    match (expr_a, expr_b) {
        (LispExp::List(x), LispExp::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(m, n)| equal_function_helper(m, n))
        }
        (x, y) => x == y,
    }
}

/// deal with variadic lambda's parameters list
/// "(a . b)" => params: vec![intern("a")], rest: Some(intern("b"));
/// "(a b c)" => params: vec![intern("a"),intern("b"),intern("c")], rest: None;
fn eval_lambda_get_paras_helper(
    my_lambda_args: &Vec<LispExp>,
) -> Result<(Vec<u64>, Option<u64>), LispErr> {
    let mut params_vec: Vec<u64> = Vec::new();
    let mut rest_params_id: Option<u64> = None;
    let mut have_dot_flag: bool = false;
    for the_lambda_arg in my_lambda_args {
        match the_lambda_arg {
            LispExp::Symbol(s) => {
                match lookup(s.clone()).as_str() {
                    "." => {
                        have_dot_flag = true;
                        continue;
                    }
                    _ => match have_dot_flag {
                        true => match rest_params_id {
                            Some(_) => return Err(LispErr::Reason(
                                "\'variadic lambda\' 's should only have one \'rest paramenter\'"
                                    .to_string(),
                            )),
                            None => rest_params_id = Some(s.clone()),
                        },
                        false => {
                            params_vec.push(s.clone());
                        }
                    },
                }
            }
            _ => {
                return Err(LispErr::Reason(
                    "\'lambda\' 's parameters must be symbol(s)".into(),
                ))
            }
        }
    }
    Ok((params_vec, rest_params_id))
}

fn eval_defmacro_get_lambda_paras_helper(
    my_defmacro_lambda_args: &Vec<LispExp>,
) -> Result<(Vec<u64>, Option<u64>), LispErr> {
    let mut params_vec: Vec<u64> = Vec::new();
    let mut rest_params_id: Option<u64> = None;
    let mut have_dot_flag: bool = false;
    for the_lambda_arg in my_defmacro_lambda_args {
        match the_lambda_arg {
            LispExp::Symbol(s) => {
                match lookup(s.clone()).as_str() {
                    "." => {
                        have_dot_flag = true;
                        continue;
                    }
                    _ => match have_dot_flag {
                        true => match rest_params_id {
                            Some(_) => return Err(LispErr::Reason(
                                "\'variadic lambda\' in \'defmacro\' should only have one \'rest paramenter\'"
                                    .to_string(),
                            )),
                            None => rest_params_id = Some(s.clone()),
                        },
                        false => {
                            params_vec.push(s.clone());
                        }
                    },
                }
            }
            _ => {
                return Err(LispErr::Reason(
                    "\'lambda\' in \'defmacro\' 's parameters must be symbol(s)".into(),
                ))
            }
        }
    }
    Ok((params_vec, rest_params_id))
}

/// quasiquote expand helper function
fn quasiquote_expand_helper(
    template: &LispExp,
    predefined_syms: &PredefinedSyms,
) -> Result<LispExp, LispErr> {
    let the_quote = predefined_syms.quote_sym;
    let the_unquote = predefined_syms.unquote;
    let the_unquote_splicing = predefined_syms.unquote_splicing;
    let the_append = intern("append");
    let the_cons = intern("cons");
    // consider replace 'if..(else if)..else' with 'match'
    match template {
        LispExp::List(contents) => {
            match contents.is_empty() {
                true => {
                    // empty list
                    return Ok(LispExp::List(vec![
                        LispExp::Symbol(the_quote),
                        LispExp::List(vec![]),
                    ]));
                }
                false => {
                    match contents[0] {
                        LispExp::Symbol(s) => {
                            // special list
                            if s == the_unquote && contents.len() == 2 {
                                return Ok(contents[1].clone());
                            } else if s == the_unquote {
                                return Err(LispErr::Reason(
                                    "\'unquote\'(\',\') should have exactly one argument".into(),
                                ));
                            } else if s == the_unquote_splicing {
                                // ,@x cannot appear at list head (it can only splice in element position)
                                // No special handling here, falls through to normal list-building logic.
                                // Note: ,@ at list head is undefined behavior in standard Lisp.
                                // This implementation treats it as a regular element, which may
                                // produce unexpected results. Avoid using ,@ at list head.
                                return Err(LispErr::Reason(
                                    "\'unquote-splicing\'(\',@\') shall not be the 1st element of the \'quasiquote\' list".into(),
                                ));
                            }
                        }
                        _ => {}
                    }
                    // oridinary list->build 'cons' chain right to left
                    // e.g. `(a ,b c)--->(cons 'a (cons b (cons 'c '())))
                    let mut result =
                        LispExp::List(vec![LispExp::Symbol(the_quote), LispExp::List(vec![])]);
                    // iterate right-to-left
                    for el in contents.iter().rev() {
                        if let LispExp::List(inner) = el {
                            if inner.len() == 2 {
                                if let LispExp::Symbol(s) = &inner[0] {
                                    if *s == the_unquote_splicing {
                                        // ,@expr -> (append expr <accumulated result>)
                                        result = LispExp::List(vec![
                                            LispExp::Symbol(the_append),
                                            inner[1].clone(),
                                            result,
                                        ]);
                                        continue;
                                    } else if *s == the_unquote {
                                        // ,expr -> (cons expr <accumulated result>)
                                        result = LispExp::List(vec![
                                            LispExp::Symbol(the_cons),
                                            inner[1].clone(),
                                            result,
                                        ]);
                                        continue;
                                    }
                                }
                            }
                        };
                        let expanded = quasiquote_expand_helper(el, predefined_syms).unwrap();
                        result = LispExp::List(vec![LispExp::Symbol(the_cons), expanded, result]);
                    }
                    return Ok(result);
                }
            }
        }
        _ => {
            return Ok(LispExp::List(vec![
                LispExp::Symbol(the_quote),
                template.clone(),
            ]));
        }
    }
}
