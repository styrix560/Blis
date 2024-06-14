use core::panic;
use std::{
    clone,
    collections::{HashMap, VecDeque},
};

use crate::Lambda;

fn insert_arguments(root: &mut Lambda, args: &mut VecDeque<Lambda>) {
    println!("inserting {:?} into {root}", args);
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

fn beta_reduction(
    name: &str,
    replacement: &Lambda,
    body: Box<Lambda>,
    variable_renames: &mut HashMap<String, String>,
) -> Lambda {
    println!("replacing {name} in {body} with {replacement}");
    match *body {
        Lambda::Value(value) => {
            let new_value = variable_renames.get(&value).unwrap_or(&value);
            if new_value == name {
                replacement.clone()
            } else {
                Lambda::val(new_value)
            }
        }
        Lambda::Definition {
            mut input,
            body,
            parameter,
        } => {
            let new_name = if variable_renames.contains_key(&input) {
                let renamed = variable_renames.get(&input).unwrap();
                if renamed == &input {
                    // this name is already defined. pick a new one
                    let mut number = 1;
                    while variable_renames.contains_key(&(input.clone() + &number.to_string())) {
                        number += 1;
                    }
                    input.clone() + &number.to_string()
                } else {
                    // this name was already renamed
                    variable_renames.get(&input).unwrap().clone()
                }
            } else {
                input.clone()
            };
            variable_renames.insert(input, new_name.clone());
            let new_body = beta_reduction(name, replacement, body, variable_renames);
            variable_renames.remove(&new_name);

            let new_parameter = parameter
                .map(|p| beta_reduction(name, replacement, p, variable_renames))
                .map(Box::new);

            Lambda::Definition {
                input: new_name,
                body: Box::new(new_body),
                parameter: new_parameter,
            }
        }
        Lambda::Call {
            function_name: input,
            parameter,
        } => {
            let input = if variable_renames.contains_key(&input) {
                variable_renames.get(&input).unwrap().clone()
            } else {
                input
            };
            assert!(
                matches!(
                    replacement,
                    Lambda::Definition {
                        input,
                        body,
                        parameter
                    }
                ) || matches!(replacement, Lambda::Value(_))
            );
            let mut new_parameter: VecDeque<Lambda> = parameter
                .into_iter()
                .map(|p| beta_reduction(name, replacement, Box::new(p), variable_renames))
                .collect();

            if input == name {
                let mut replacement = replacement.clone();
                insert_arguments(&mut replacement, &mut new_parameter);
                replacement
            } else {
                Lambda::Call {
                    function_name: input,
                    parameter: new_parameter,
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
        let mut map = HashMap::new();
        if let Lambda::Value(value) = parameter.as_ref().unwrap().as_ref() {
            map.insert(value.clone(), value.clone());
        }
        return beta_reduction(&input, &parameter.unwrap(), body, &mut map);
    }
    unreachable!()
}

pub(crate) fn full_reduce(mut root: Lambda) -> Lambda {
    for _ in 0..1000 {
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
    root
}

#[cfg(test)]
mod tests {

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
            Lambda::call("f", vec![Lambda::val("y")]),
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
                Lambda::call("a", vec![Lambda::val("b")]),
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

    #[test]
    fn name_collision() {
        // y(x(y.x)).x(x) => y(x1(y.x1)).x(x) => x1(x(x).x1) => x1(x1)
        let lambda = Lambda::def(
            "y",
            Lambda::def("x", Lambda::call("x", vec![Lambda::val("y")]), None),
            Some(Lambda::val("x")),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::def("x1", Lambda::val("x1"), None))
    }
}
