/// break src strs into tokens
pub fn tokenize(input: &str) -> Vec<String> {
    input
        .replace("(", " ( ")
        .replace(")", " ) ")  // deal with the parentheses problems
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
