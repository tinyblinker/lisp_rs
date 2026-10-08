/// break src strs into tokens
pub fn tokenize(input: &str) -> Vec<String> {
    let without_comments = input
        .lines()
        .map(|line| match line.find(";") {
            Some(pos) => &line[..pos],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    without_comments
        .replace("(", " ( ")
        .replace(")", " ) ") // deal with the parentheses problems
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
