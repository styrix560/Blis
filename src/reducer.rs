use core::panic;
use std::{
    any, clone,
    collections::{HashMap, VecDeque},
    ops::Deref,
};

use crate::Lambda;

fn insert_arguments(root: &mut Lambda, args: &mut VecDeque<Lambda>) {
    println!("inserting {:?} into {root}", args);
    if args.is_empty() {
        return;
    }
    match root {
        Lambda::Value(name) => {
            *root = Lambda::call(name, args.iter_mut().map(|a| a.clone()).collect())
        }
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
            parameters: parameter,
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
            input,
            body,
            parameter,
        } => {
            let new_name = if variable_renames.contains_key(&input) {
                // TODO: i dont think this should work
                let mut renamed = variable_renames.get(&input).unwrap().clone();
                if renamed == input {
                    // this name is already defined. pick a new one

                    while variable_renames.contains_key(&renamed) {
                        renamed += "_";
                    }
                    renamed
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
            parameters: parameter,
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
                    parameters: new_parameter,
                }
            }
        }
    }
}

fn get_bound_variables<'a>(root: &'a Lambda, variables: &mut Vec<&'a str>) {
    match root {
        Lambda::Value(_) => {}
        Lambda::Definition {
            input,
            body,
            parameter,
        } => {
            variables.push(input);
            get_bound_variables(body.as_ref(), variables);
            if let Some(parameter) = parameter {
                get_bound_variables(parameter.as_ref(), variables);
            }
        }
        Lambda::Call {
            function_name,
            parameters: parameter,
        } => {
            let _ = parameter.iter().map(|p| get_bound_variables(p, variables));
        }
    }
}

fn reduce(root: Lambda) -> Lambda {
    println!("reducing {}", root);
    if let Lambda::Definition {
        input,
        body,
        parameter,
    } = root
    {
        assert!(parameter.is_some());
        let parameter = parameter.unwrap();
        let mut map = HashMap::new();
        let mut variables = Vec::new();
        get_bound_variables(parameter.as_ref(), &mut variables);
        for p in variables {
            map.insert(p.to_owned(), p.to_owned());
        }
        return beta_reduction(&input, &parameter, body, &mut map);
    }
    unreachable!()
}

fn find_reducible(root: Lambda) -> Result<Lambda, Lambda> {
    match root {
        Lambda::Value(_) => Err(root),
        Lambda::Definition {
            input,
            body,
            parameter,
        } => {
            if parameter.is_none() {
                let new_body = find_reducible(*body);
                if new_body.is_ok() {
                    Ok(Lambda::def(
                        &input,
                        new_body.unwrap(),
                        parameter.map(|p| *p),
                    ))
                } else {
                    Err(Lambda::def(
                        &input,
                        new_body.unwrap_err(),
                        parameter.map(|p| *p),
                    ))
                }
            } else {
                Ok(reduce(Lambda::def(&input, *body, parameter.map(|p| *p))))
            }
        }
        Lambda::Call {
            ref function_name,
            parameters,
        } => {
            let iter = parameters.into_iter().map(find_reducible);
            let mut new_parameters = Vec::new();
            let mut any_reduced = false;
            for p in iter {
                if let Ok(result) = p {
                    any_reduced = true;
                    new_parameters.push(result);
                } else {
                    new_parameters.push(p.unwrap_err());
                }
            }
            if any_reduced {
                Ok(Lambda::call(function_name, new_parameters))
            } else {
                Err(Lambda::call(function_name, new_parameters))
            }
        }
    }
}

pub(crate) fn full_reduce(mut root: Lambda) -> Lambda {
    for _ in 0..1000 {
        println!("{}", root);
        let result = find_reducible(root);
        if result.is_err() {
            return result.unwrap_err();
        }
        root = result.unwrap();
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
            Lambda::def("x", Lambda::call("y", vec![Lambda::val("x")]), None),
            Some(Lambda::def("x", Lambda::val("x"), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::def("x_", Lambda::val("x_"), None))
    }

    #[test]
    fn complex_name_collisions() {
        // a(b(c(a.(b.(c.5))))).a(a).a(a).a(a)
        let lambda = Lambda::def(
            "a",
            Lambda::def(
                "b",
                Lambda::def(
                    "c",
                    Lambda::call(
                        "a",
                        vec![Lambda::call(
                            "b",
                            vec![Lambda::call("a", vec![Lambda::val("5")])],
                        )],
                    ),
                    Some(Lambda::def("a", Lambda::val("a"), None)),
                ),
                Some(Lambda::def("a", Lambda::val("a"), None)),
            ),
            Some(Lambda::def("a", Lambda::val("a"), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val("5"))
    }
}
