use core::panic;
use std::collections::VecDeque;

use crate::Lambda;

fn insert_arguments(root: &mut Lambda, args: &mut VecDeque<Lambda>) {
    if args.is_empty() {
        return;
    }
    match root {
        Lambda::Value(_) => panic!("Cannot call value"),
        Lambda::Definition {
            input,
            body,
            parameter,
        } => {
            if parameter.is_none() {
                *parameter = args.pop_front().map(Box::new);
            }
            insert_arguments(body, args);
        }
        Lambda::Call {
            function_name,
            parameter,
        } => {
            parameter.append(args);
        }
    }
}

fn replace(name: &str, replacement: &Lambda, body: Box<Lambda>) -> Lambda {
    match *body {
        Lambda::Value(value) => {
            if value == name {
                replacement.clone()
            } else {
                Lambda::val(&value)
            }
        }
        Lambda::Definition {
            input,
            body,
            parameter,
        } => {
            assert!(input != name);
            let new_body = replace(name, replacement, body);
            let new_parameter = parameter
                .map(|p| replace(name, replacement, p))
                .map(Box::new);

            Lambda::Definition {
                input,
                body: Box::new(new_body),
                parameter: new_parameter,
            }
        }
        Lambda::Call {
            function_name: input,
            mut parameter,
        } => {
            assert!(matches!(
                replacement,
                Lambda::Definition {
                    input,
                    body,
                    parameter
                }
            ));
            if input == name {
                let mut replacement = replacement.clone();
                insert_arguments(&mut replacement, &mut parameter);
                replacement
            } else {
                Lambda::Call {
                    function_name: input,
                    parameter,
                }
            }
        }
    }
}

fn reduce(root: Lambda) -> Lambda {
    if let Lambda::Definition {
        input,
        body,
        parameter,
    } = root
    {
        assert!(parameter.is_some());
        return replace(&input, &parameter.unwrap(), body);
    }
    unreachable!()
}

pub(crate) fn full_reduce(mut root: Lambda) -> Lambda {
    loop {
        println!("{}", root);
        match &root {
            Lambda::Value(_) => return root,
            Lambda::Definition {
                input,
                body,
                parameter,
            } => {
                if parameter.is_some() {
                    root = reduce(root)
                } else {
                    return root;
                }
            }
            Lambda::Call {
                function_name,
                parameter,
            } => panic!("Unresolved function"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{reducer::full_reduce, Lambda};

    #[test]
    fn no_reduction() {
        let lambda = Lambda::def("a", Lambda::val("a"), None);
        let reduced = full_reduce(lambda.clone());
        assert_eq!(lambda, reduced);
    }

    #[test]
    fn simple_reduction() {
        let lambda = Lambda::def(
            "f",
            Lambda::call("f", VecDeque::from(vec![Lambda::val("y")])),
            Some(Lambda::def("x", Lambda::val("x"), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val("y"))
    }

    #[test]
    fn complex_reduction() {
        // a(b(a.b)).(c(d(d)).5).3
        let lambda = Lambda::def(
            "a",
            Lambda::def(
                "b",
                Lambda::call("a", VecDeque::from(vec![Lambda::val("b")])),
                Some(Lambda::val("3")),
            ),
            Some(Lambda::def(
                "c",
                Lambda::def("d", Lambda::val("d"), None),
                Some(Lambda::val("5")),
            )),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val("3"));
    }
}
