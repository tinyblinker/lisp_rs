/// break src strs into tokens
pub fn tokenize(input: &str) -> Vec<String> {
    input.split_whitespace().map(|s| s.to_string()).collect()
}
