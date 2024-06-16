pub(crate) fn compile(text: &str) -> String {
    let mut lets = Vec::new();
    let mut index = 0;
    while text[index..].trim().starts_with("let") {
        let text = &text[index..];
        let mut s = text.trim().split(';').next().unwrap().split(' ');
        let _ = s.next(); // skip let
        let name = s.next().unwrap();
        let body: String = s.collect();
        lets.push((name, body));
        index += text
            .find(';')
            .expect("Expect semicolon after let-definition")
            + 1;
    }
    let text = &text[index..];
    let mut string = String::from(text);
    for (name, body) in lets.into_iter().rev() {
        string = format!("{name}({string}).{body}");
    }
    string
}

#[cfg(test)]
mod tests {
    use crate::parser::remove_whitespace;

    use super::compile;

    #[test]
    fn no_let() {
        let text = "a(a.5).a(a)";
        let compiled = compile(text);
        assert_eq!(compiled, text);
    }

    #[test]
    fn single_let() {
        let text = "let f a(a.5);
        f.a(a)";
        let compiled = remove_whitespace(&compile(text));
        assert_eq!(compiled, "f(f.a(a)).a(a.5)")
    }

    #[test]
    fn double_let() {
        let text = "
        let f a(a.5);
        let g a(a.3);
        f.g";
        let compiled = remove_whitespace(&compile(text));
        assert_eq!(compiled, "f(g(f.g).a(a.3)).a(a.5)")
    }
}
