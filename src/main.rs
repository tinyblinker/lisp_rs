use std::io::{self, BufRead, Stdin, Write};

use my_lisp::{
    env::LispEnv,
    interpreter::{create_default_env, eval},
    lexer::tokenize,
    parser::parse,
    LispErr, LispExp,
};

/// read input that may span multiple lines
fn read_input(stdin: &Stdin) -> Option<String> {
    let mut buffer = String::new();
    let mut depth: i32 = 0;
    let mut got_input = false;
    loop {
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                if got_input {
                    break;
                } else {
                    return None;
                }
            }
            Ok(_) => {
                got_input = true;
                buffer.push_str(&line);
                let mut in_string = false; // whether the char is in a LispExp::String
                let mut escape = false; // whether the char is demostrate by '\'
                for ch in line.chars() {
                    if in_string {
                        match ch {
                            '\\' => escape = !escape,
                            '"' => in_string = false,
                            _ => escape = false,
                        }
                    } else {
                        match ch {
                            '"' => in_string = true,
                            '(' => depth += 1,
                            ')' => depth -= 1,
                            _ => {}
                        }
                    }
                }
                // get closed or unclosed exprs, return immediatly
                if depth <= 0 {
                    break;
                }
                print!("...");
                io::stdout().flush().unwrap();
            }
            Err(_) => break,
        }
    }
    Some(buffer)
}

fn eval_input(input: &str, env: &mut LispEnv) -> Result<String, String> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Ok("nil".to_string());
    }
    let mut remaining: &[&str] = &tokens;
    let mut results: Vec<LispExp> = Vec::new();
    while !remaining.is_empty() {
        let (exp, rest) = parse(remaining).map_err(|e| match e {
            LispErr::Reason(msg) => msg,
        })?;
        remaining = rest;
        match eval(&exp, env) {
            Ok(val) => {
                results.push(val);
            }
            Err(e) => match e {
                LispErr::Reason(msg) => return Err(msg),
            },
        };
    }
    if results.is_empty() {
        Ok("nil".to_string())
    } else if results.len() == 1 {
        Ok(format!("{}", results[0]))
    } else {
        Ok(results
            .iter()
            .map(|r| format!("{}", r))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}
fn main() {
    // [TODO] write a benchmark to bench the interpreter
    println!("==Heart!Heart!Meow!==");
    println!("<Lisp-rs REPL v0.0.1>");
    println!("Type :help for help, :q to quit, Ctrl+D to exit\n");
    let mut env = create_default_env();
    let stdin = std::io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let input = match read_input(&stdin) {
            Some(s) => s.trim().to_string(),
            None => {
                println!("GoodBye my friend!");
                break;
            }
        };
        if input.is_empty() {
            continue;
        } else if input.starts_with(':') {
            match input.as_str() {
                ":q" | ":quit" | ":exit" => {
                    println!("GoodBye my friend!");
                    break;
                }
                ":help" => {
                    println!("Special forms: if define lambda begin set! let cond and or quote");
                    println!(
                        "Built-in functions: + - * / = > < >= <= not list cons car cdr cadr caddr"
                    );
                    println!("Commands: :q quit, :help help");
                    continue;
                }
                _ => {
                    println!("Unknown command: {}", input);
                    continue;
                }
            }
        }
        match eval_input(&input, &mut env) {
            Ok(repl_result) => println!("{}", repl_result),
            Err(err) => println!("Error:{}", err),
        }
    }
}
