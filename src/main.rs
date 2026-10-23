use std::io::{self, BufRead, Stdin, Write};

use my_lisp::{env::LispEnv, interpreter::create_default_env, LispErr, LispExp};

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
                let mut in_string = false;
                let mut escape = false;
                for ch in line.chars() {
                    match ch {
                        '"' if !escape => {
                            in_string = !in_string;
                        }
                        '\\' if in_string => {
                            escape = !escape;
                        }
                        '(' if in_string => {
                            depth += 1;
                        }
                        ')' if in_string => {
                            depth -= 1;
                        }
                        _ => {
                            escape = false;
                        }
                    }
                }
                if depth <= 0 {
                    break;
                }
                print!("... ");
                io::stdout().flush().unwrap();
                todo!();
            }
            Err(_) => break,
        }
    }
    todo!();
}

fn eval_input(input: &String, env: &mut LispEnv) -> Result<LispExp, LispErr> {
    todo!();
}

fn main() {
    // [TODO] write a benchmark to bench the interpreter
    println!("==Heart!Heart!Meow!==");
    println!("<Lisp-rs REPL v0.0.1>");
    println!("Type:help for help, :q to quit, Ctrl+D to exit\n");
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
