use std::collections::VecDeque;

use crate::{helpers::find_block_end, Lambda};

#[derive(Debug)]
enum ParseType {
    Value,
    Definition,
    Call,
}

pub(crate) struct Binder {
    pub(crate) global_bindings: Vec<String>,
    pub(crate) bindings_stack: Vec<usize>,
}

impl Binder {
    fn new() -> Self {
        Binder {
            global_bindings: Vec::new(),
            bindings_stack: Vec::new(),
        }
    }
    pub(crate) fn get_index(&mut self) -> usize {
        self.global_bindings.len()
    }
    pub(crate) fn find_index(&self, value: &str) -> Option<usize> {
        self.global_bindings
            .iter()
            .enumerate()
            .filter(|(index, _)| self.bindings_stack.contains(index))
            .find(|(_, binding)| *binding == value)
            .map(|p| p.0)
    }
    pub(crate) fn new_binding(&mut self, name: String) -> usize {
        let index = self.get_index();
        self.bindings_stack.push(index);
        self.global_bindings.push(name);
        index
    }
    fn pop_binding(&mut self) {
        self.bindings_stack.pop();
    }
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

fn parse_call(text: &str, arguments: &mut VecDeque<Lambda>, binder: &mut Binder) -> Lambda {
    let name_end = text.find('.').unwrap();
    let name = &text[..name_end];
    let mut args = parse_arguments(&text[name_end..], binder);
    args.append(arguments);
    arguments.append(&mut args);

    Lambda::new_call(
        name,
        std::mem::take(arguments).into_iter().collect(),
        binder,
    )
}

fn parse_arguments(mut text: &str, binder: &mut Binder) -> VecDeque<Lambda> {
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

        let arg = parse(arg_text, &mut VecDeque::new(), binder);
        args.push_back(arg);
        text = &text[arg_end..];
    }
    args
}

fn parse_definition(text: &str, arguments: &mut VecDeque<Lambda>, binder: &mut Binder) -> Lambda {
    let name_end = text.find('(').unwrap();
    let name = &text[..name_end];
    assert!(
        !binder
            .bindings_stack
            .iter()
            .map(|index| binder.global_bindings[*index].as_str())
            .collect::<Vec<&str>>()
            .contains(&name),
        "that name is already defined: {name}",
    );

    let body_end = find_block_end(text).unwrap();

    let parameter = if body_end + 1 < text.len() {
        let mut iter = parse_arguments(&text[body_end + 1..], binder).into_iter();
        let argument = iter.next();
        arguments.extend(iter);
        argument
    } else {
        arguments.pop_front()
    };

    let name_index = binder.new_binding(name.to_owned());
    let body = parse(&text[name_end + 1..body_end], arguments, binder);
    binder.pop_binding();

    Lambda::def(name_index, body, parameter)
}

fn parse(text: &str, arguments: &mut VecDeque<Lambda>, binder: &mut Binder) -> Lambda {
    if text.starts_with('(') {
        let end = find_block_end(text).unwrap();
        if end < text.len() - 1 {
            assert!(text[end + 1..].starts_with('.'));
            let mut args = parse_arguments(&text[end + 1..], binder);
            arguments.append(&mut args);
        }
        return parse(&text[1..end], arguments, binder);
    }
    let parse_type = get_type(text);

    match parse_type {
        ParseType::Value => Lambda::new_var(text, binder),
        ParseType::Definition => parse_definition(text, arguments, binder),
        ParseType::Call => parse_call(text, arguments, binder),
    }
}

pub(crate) fn remove_whitespace(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
}

pub(crate) fn parse_program(text: &str) -> (Lambda, Vec<String>) {
    let mut arguments = VecDeque::new();
    let mut binder = Binder::new();
    (
        parse(&remove_whitespace(text), &mut arguments, &mut binder),
        binder.global_bindings,
    )
}

#[cfg(test)]
mod tests {

    use crate::{parser::parse_program, Lambda};

    #[test]
    fn parse_value() {
        let text = "hi".to_string();
        let (result, _bindings) = parse_program(&text);
        assert_eq!(result, Lambda::Variable(0))
    }

    #[test]
    fn parenthesis_around_value() {
        let text = "(hi)".to_string();
        let (result, _bindings) = parse_program(&text);
        assert_eq!(result, Lambda::Variable(0))
    }

    #[test]
    fn function_definition() {
        let text = "a(a)".to_string();
        let (result, bindings) = parse_program(&text);
        assert_eq!(result, Lambda::def(0, Lambda::var(0), None), "{bindings:?}");
    }

    #[test]
    fn nested_function_def() {
        let text = "a(b(c(a)))".to_string();
        let (result, _bindings) = parse_program(&text);
        assert_eq!(
            result,
            Lambda::def(
                0,
                Lambda::def(1, Lambda::def(2, Lambda::var(0), None), None),
                None
            )
        )
    }

    #[test]
    fn immediate_call() {
        let text = "a(a).5".to_string();
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["5", "a"]);
        assert_eq!(result, Lambda::def(1, Lambda::var(1), Some(Lambda::var(0))));
    }

    #[test]
    fn double_call() {
        let text = "a(b(a)).5.3".to_string();
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["5", "3", "a", "b"]);
        assert_eq!(
            result,
            Lambda::def(
                2,
                Lambda::def(3, Lambda::var(2), Some(Lambda::var(1))),
                Some(Lambda::var(0))
            ),
            "{bindings:?}"
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
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["c", "5", "a", "b"]);
        assert_eq!(
            result,
            Lambda::def(
                2,
                Lambda::def(
                    3,
                    Lambda::call(2, vec![Lambda::var(3)]),
                    Some(Lambda::var(1))
                ),
                Some(Lambda::def(0, Lambda::var(0), None))
            ),
            "{bindings:?}"
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
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["d", "e", "5", "3", "a", "b", "c"]);
        assert_eq!(
            result,
            Lambda::def(
                4,
                Lambda::def(
                    5,
                    Lambda::def(
                        6,
                        Lambda::call(4, vec![Lambda::var(5), Lambda::var(6)]),
                        Some(Lambda::var(3))
                    ),
                    Some(Lambda::var(2))
                ),
                Some(Lambda::def(0, Lambda::def(1, Lambda::var(1), None), None))
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
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["d", "e", "5", "3", "a", "b", "c"]);
        assert_eq!(
            result,
            Lambda::def(
                4,
                Lambda::def(
                    5,
                    Lambda::def(
                        6,
                        Lambda::call(4, vec![Lambda::var(5), Lambda::var(6)]),
                        Some(Lambda::var(3))
                    ),
                    Some(Lambda::var(2))
                ),
                Some(Lambda::def(0, Lambda::def(1, Lambda::var(1), None), None))
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
        let (result, bindings) = parse_program(&text);
        assert_eq!(bindings, vec!["5", "3", "a", "b", "7", "c"]);
        assert_eq!(
            result,
            Lambda::def(
                2,
                Lambda::def(
                    3,
                    Lambda::def(5, Lambda::var(3), Some(Lambda::var(4))),
                    Some(Lambda::var(1))
                ),
                Some(Lambda::var(0))
            )
        )
    }

    #[test]
    fn naming_duplication() {
        let text = "a(a.5).a(a)";
        let (result, bindings) = parse_program(text);
        assert_eq!(bindings, vec!["a", "a", "5"]);
        assert_eq!(
            result,
            Lambda::def(
                1,
                Lambda::call(1, vec![Lambda::var(2)]),
                Some(Lambda::def(0, Lambda::var(0), None))
            )
        )
    }

    #[test]
    fn calling_with_itself() {
        let text = "a(a.a).a(a)";
        let (result, bindings) = parse_program(text);
        assert_eq!(bindings, vec!["a", "a"]);
        assert_eq!(
            result,
            Lambda::def(
                1,
                Lambda::call(1, vec![Lambda::var(1)]),
                Some(Lambda::def(0, Lambda::var(0), None))
            )
        )
    }

    #[test]
    #[should_panic]
    fn naming_collision() {
        let text = "a(a(a))";
        parse_program(text);
    }
}
