use core::panic;
use std::{cmp::min, collections::VecDeque, mem, ops::Index};

use crate::Lambda;

#[derive(Debug)]
enum ParseType {
    Value,
    Definition,
    Call,
}

fn find_block_end(text: &str) -> Option<usize> {
    let mut stack = 0;
    for (index, char) in text.char_indices() {
        if char == '(' {
            stack += 1;
        }
        if char == ')' {
            stack -= 1;
            if stack < 0 {
                panic!()
            }
            if stack == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn get_type(text: &str) -> ParseType {
    let call_start = text.find('.');
    let definition_start = text.find('(');
    // otherwise it would be a value
    if call_start.is_none() && definition_start.is_none() {
        return ParseType::Value;
    }
    if definition_start.unwrap_or(usize::MAX) < call_start.unwrap_or(usize::MAX) {
        return ParseType::Definition;
    }
    ParseType::Call
}

fn parse_call(text: &str, arguments: &mut VecDeque<Lambda>) -> Lambda {
    let name_end = text.find('.').unwrap();
    let name = &text[..name_end];
    let mut args = parse_arguments(&text[name_end..]);
    args.append(arguments);
    arguments.append(&mut args);

    Lambda::Call {
        function_name: name.to_string(),
        parameter: std::mem::take(arguments),
    }
}

fn parse_arguments(mut text: &str) -> VecDeque<Lambda> {
    let mut args = VecDeque::new();

    while !text.is_empty() {
        assert!(text.starts_with('.'));
        text = &text[1..];

        let call_end = text.find('.');
        let block_start = text.find('(').unwrap_or(usize::MAX);
        let arg_end = if block_start < call_end.unwrap_or(usize::MAX) {
            find_block_end(text).unwrap() + 1
        } else if call_end.is_some() {
            call_end.unwrap()
        } else {
            text.len()
        };

        let arg_text = &text[..arg_end];

        // println!(
        //     "callend: {:?}\nblock_start: {}\narg_end: {}\narg_text: {}\n-----",
        //     call_end, block_start, arg_end, arg_text
        // );
        let arg = parse(arg_text, &mut VecDeque::new());
        args.push_back(arg);
        text = &text[arg_end..];
    }
    args
}

fn parse_definition(text: &str, arguments: &mut VecDeque<Lambda>) -> Lambda {
    let name_end = text.find('(').unwrap();
    let name = &text[..name_end];

    let body_end = find_block_end(text).unwrap();
    let parameter = if body_end + 1 < text.len() {
        let mut iter = parse_arguments(&text[body_end + 1..]).into_iter();
        let argument = iter.next();
        arguments.extend(iter);
        argument
    } else {
        arguments.pop_front()
    };
    let body = parse(&text[name_end + 1..body_end], arguments);
    println!("{}", &text[body_end + 1..]);
    Lambda::def(name, body, parameter)
}

fn parse(text: &str, arguments: &mut VecDeque<Lambda>) -> Lambda {
    if text.starts_with('(') {
        let end = find_block_end(text).unwrap();
        println!("unpacking {}", text);
        if end < text.len() - 1 {
            println!("got some extra args");
            assert!(text[end + 1..].starts_with('.'));
            let mut args = parse_arguments(&text[end + 1..]);
            println!("extra args {:?}", args);
            arguments.append(&mut args);
        }
        return parse(&text[1..end], arguments);
    }
    let parse_type = get_type(text);

    println!("{}: {:?}", text, parse_type);

    match parse_type {
        ParseType::Value => Lambda::Value(text.to_string()),
        ParseType::Definition => parse_definition(text, arguments),
        ParseType::Call => parse_call(text, arguments),
    }
}

pub(crate) fn parse_program(text: &str) -> Lambda {
    let mut arguments = VecDeque::new();
    parse(
        &text
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>(),
        &mut arguments,
    )
}

#[cfg(test)]
mod tests {

    use crate::{parser::parse_program, Lambda};

    #[test]
    fn parse_value() {
        let text = "hi".to_string();
        let result = parse_program(&text);
        assert_eq!(result, Lambda::Value("hi".to_string()))
    }

    #[test]
    fn parenthesis_around_value() {
        let text = "(hi)".to_string();
        let result = parse_program(&text);
        assert_eq!(result, Lambda::Value("hi".to_string()))
    }

    #[test]
    fn function_definition() {
        let text = "a(a)".to_string();
        let result = parse_program(&text);
        assert_eq!(result, Lambda::def("a", Lambda::val("a"), None));
    }

    #[test]
    fn nested_function_definition() {
        let text = "a(b(c(a)))".to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def("b", Lambda::def("c", Lambda::val("a"), None), None),
                None
            )
        )
    }

    #[test]
    fn immediate_call() {
        let text = "a(a).5".to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def("a", Lambda::val("a"), Some(Lambda::val("5")))
        );
    }

    #[test]
    fn double_call() {
        let text = "a(b(a)).5.3".to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def("b", Lambda::val("a"), Some(Lambda::val("3"))),
                Some(Lambda::val("5"))
            )
        )
    }

    #[test]
    fn calling_of_unresolved_function() {
        let text = "
        a(
            b(
                a.b
            )
        ).c(c).5
        "
        .to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def(
                    "b",
                    Lambda::call("a", vec![Lambda::val("b")]),
                    Some(Lambda::val("5"))
                ),
                Some(Lambda::def("c", Lambda::val("c"), None))
            )
        )
    }

    #[test]
    fn nested_unresolved_call() {
        let text = "
        a(
            b(
                c(
                    a.b.c
                )
            )
        ).d(e(e)).5.3
        "
        .to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def(
                    "b",
                    Lambda::def(
                        "c",
                        Lambda::call("a", vec![Lambda::val("b"), Lambda::val("c")]),
                        Some(Lambda::val("3"))
                    ),
                    Some(Lambda::val("5"))
                ),
                Some(Lambda::def(
                    "d",
                    Lambda::def("e", Lambda::val("e"), None),
                    None
                ))
            ),
            "{}",
            result
        )
    }

    #[test]
    fn parenthesises() {
        let text = "
        (a(
            b(
                c(
                    ((a.b).c)
                )
            )
        )).(d((e((e))))).((5)).3
        "
        .to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def(
                    "b",
                    Lambda::def(
                        "c",
                        Lambda::call("a", vec![Lambda::val("b"), Lambda::val("c")]),
                        Some(Lambda::val("3"))
                    ),
                    Some(Lambda::val("5"))
                ),
                Some(Lambda::def(
                    "d",
                    Lambda::def("e", Lambda::val("e"), None),
                    None
                ))
            ),
            "{}",
            result
        )
    }

    #[test]
    fn double_late_call() {
        let text = "
        (
            a(
                b(
                    c(b).7
                )
            )
        ).5.3
        "
        .to_string();
        let result = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                "a",
                Lambda::def(
                    "b",
                    Lambda::def("c", Lambda::val("b"), Some(Lambda::val("7"))),
                    Some(Lambda::val("3"))
                ),
                Some(Lambda::val("5"))
            )
        )
    }
}
