pub fn tokenize(input: &str) -> Vec<String> {
    let stripped_comments = input
        .lines()
        .map(|line| match line.find(";") {
            Some(pos) => &line[0..pos],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    stripped_comments
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
