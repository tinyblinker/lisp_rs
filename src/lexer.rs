/// tokenize() is the "Lexer"
/// break src strs into tokens
pub fn tokenize(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    while let Some((index, char)) = chars.next() {
        match char {
            '(' => tokens.push(&input[index..=index]),
            ')' => tokens.push(&input[index..=index]),
            '\'' => tokens.push(&input[index..=index]),
            '"' => {
                let start = index;
                while let Some((i, c)) = chars.next() {
                    if c == '"' {
                        let end = i;
                        tokens.push(&input[start..=end]);
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        continue;
                    }
                }
            }
            ';' => {
                while let Some((_, c)) = chars.peek() {
                    if *c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            c if c.is_whitespace() => {}
            _ => {
                // normal symbol
                let start = index;
                while let Some((_, c)) = chars.peek() {
                    if c.is_whitespace() || *c == '(' || *c == ')' || *c == '"' || *c == '\'' {
                        break;
                    }
                    chars.next();
                }
                let end = chars.peek().map_or(input.len(), |(j, _)| *j);
                tokens.push(&input[start..end]);
            }
        }
    }
    tokens
}
