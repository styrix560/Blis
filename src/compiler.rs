use regex::Regex;

use crate::{helpers::find_block_end, parser::remove_whitespace};

fn replace_lets(text: &str) -> String {
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

pub(crate) fn compile(text: &str) -> String {
    let after_lets_replaced = replace_lets(text);
    let without_whitespace = remove_whitespace(&after_lets_replaced);
    replace_comma_definition(&without_whitespace)
}

fn replace_comma_definition(text: &str) -> String {
    let mut string = String::new();
    let regex = Regex::new("[^(),.]+(,[^(),.]+)+").unwrap();
    let find = regex.find(text);
    if find.is_none() {
        return text.to_string();
    }
    let find = find.unwrap();
    let start = find.start();
    println!("here {}", &text[..start]);
    string += &text[..start];
    let args_end = find.end();
    println!("args: {}", &text[start..args_end]);
    let args = &text[start..args_end].split(',').collect::<Vec<&str>>();
    let number_of_args = args.len();
    let body_end = args_end + find_block_end(&text[args_end..]).unwrap();

    let body = &text[args_end + 1..body_end];
    for &arg in args {
        println!("arg: {arg}");
        string += arg;
        string += "(";
    }
    println!("body: {}", body);
    string += &replace_comma_definition(body);
    for _ in 0..number_of_args {
        string += ")";
    }
    string += &text[body_end + 1..];
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

    #[test]
    fn simple_comma_definition() {
        let text = "
            a,b(b).5.3
        ";
        let compiled = remove_whitespace(&compile(text));
        assert_eq!(compiled, "a(b(b)).5.3");
    }

    #[test]
    fn nested_comma_definition() {
        let text = "
            w(a,b(c,d(d).7).5.3)
        ";
        let compiled = remove_whitespace(&compile(text));
        assert_eq!(compiled, "w(a(b(c(d(d)).7)).5.3)");
    }
}
