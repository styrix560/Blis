use std::collections::VecDeque;

use crate::Lambda;

fn insert_arguments(root: &mut Lambda, args: &mut VecDeque<Lambda>) {
    if args.is_empty() {
        return;
    }
    match root {
        Lambda::Value(name) => {
            *root = Lambda::call(*name, args.iter_mut().map(|a| a.clone()).collect());
        }
        Lambda::Definition {
            name_index: _,
            body,
            parameter,
        } => {
            if parameter.is_none() {
                *parameter = args.pop_front().map(Box::new);
            }
            insert_arguments(body, args);
        }
        Lambda::Call {
            name_index: _,
            parameters: parameter,
        } => {
            parameter.append(args);
        }
    }
}

fn replace(name: usize, replacement: &Lambda, body: Box<Lambda>) -> Lambda {
    // println!("reducing {body} with {name} -> {replacement}");
    match *body {
        Lambda::Value(value) => {
            if value == name {
                replacement.clone()
            } else {
                *body
            }
        }
        Lambda::Definition {
            name_index,
            body,
            parameter,
        } => {
            // assert_ne!(name, name_index);
            let new_body = replace(name, replacement, body);

            let new_parameter = parameter
                .map(|p| replace(name, replacement, p))
                .map(Box::new);

            Lambda::Definition {
                name_index,
                body: Box::new(new_body),
                parameter: new_parameter,
            }
        }
        Lambda::Call {
            name_index,
            parameters,
        } => {
            assert!(
                matches!(
                    replacement,
                    Lambda::Definition {
                        name_index: _,
                        body: _,
                        parameter: _
                    }
                ) || matches!(replacement, Lambda::Value(_))
            );
            let mut new_parameter: VecDeque<Lambda> = parameters
                .into_iter()
                .map(|p| replace(name, replacement, Box::new(p)))
                .collect();

            if name_index == name {
                let mut replacement = replacement.clone();
                insert_arguments(&mut replacement, &mut new_parameter);
                replacement
            } else {
                Lambda::call(name_index, new_parameter.into_iter().collect())
            }
        }
    }
}

fn reduce(root: Lambda) -> Lambda {
    if let Lambda::Definition {
        name_index,
        body,
        parameter,
    } = root
    {
        assert!(parameter.is_some());
        let parameter = parameter.unwrap();
        return replace(name_index, &parameter, body);
    }
    unreachable!()
}

fn find_reducible(root: Lambda) -> Result<Lambda, Lambda> {
    match root {
        Lambda::Value(_) => Err(root),
        Lambda::Definition {
            name_index,
            body,
            parameter,
        } => {
            if parameter.is_none() {
                let new_body = find_reducible(*body);
                if let Ok(new_body) = new_body {
                    Ok(Lambda::def(name_index, new_body, parameter.map(|p| *p)))
                } else {
                    Err(Lambda::def(
                        name_index,
                        new_body.unwrap_err(),
                        parameter.map(|p| *p),
                    ))
                }
            } else {
                Ok(reduce(Lambda::def(
                    name_index,
                    *body,
                    parameter.map(|p| *p),
                )))
            }
        }
        Lambda::Call {
            name_index: function_name,
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
        println!("{root}");
        let result = find_reducible(root);
        if let Err(result) = result {
            return result;
        }
        root = result.unwrap();
    }
    root
}

#[cfg(test)]
mod tests {

    use core::panic;

    use crate::{reducer::full_reduce, Lambda};

    #[test]
    fn no_reduction() {
        // a(a)
        let lambda = Lambda::def(0, Lambda::val(0), None);
        let reduced = full_reduce(lambda.clone());
        assert_eq!(lambda, reduced);
    }

    #[test]
    fn simple_reduction() {
        let lambda = Lambda::def(
            0,
            Lambda::call(0, vec![Lambda::val(1)]),
            Some(Lambda::def(2, Lambda::val(2), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val(1))
    }

    #[test]
    fn complex_reduction() {
        // a(b(a.b)).(c(d(d)).5).3
        let lambda = Lambda::def(
            0,
            Lambda::def(
                1,
                Lambda::call(0, vec![Lambda::val(1)]),
                Some(Lambda::val(2)),
            ),
            Some(Lambda::def(
                3,
                Lambda::def(4, Lambda::val(4), None),
                Some(Lambda::val(5)),
            )),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val(2));
    }

    #[test]
    fn name_collision() {
        // 0(2(0.2)).1(1) => 2(1(1).2) => 2(2)
        let lambda = Lambda::def(
            0,
            Lambda::def(2, Lambda::call(0, vec![Lambda::val(2)]), None),
            Some(Lambda::def(1, Lambda::val(1), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::def(2, Lambda::val(2), None))
    }

    #[test]
    fn complex_name_collisions() {
        // a(b(c(a.(b.(c.5))))).a(a).a(a).a(a)
        let lambda = Lambda::def(
            0,
            Lambda::def(
                1,
                Lambda::def(
                    2,
                    Lambda::call(
                        0,
                        vec![Lambda::call(1, vec![Lambda::call(0, vec![Lambda::val(3)])])],
                    ),
                    Some(Lambda::def(6, Lambda::val(6), None)),
                ),
                Some(Lambda::def(5, Lambda::val(5), None)),
            ),
            Some(Lambda::def(4, Lambda::val(4), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::val(3))
    }

    #[test]
    fn calling_with_itself() {
        let lambda = Lambda::def(
            0,
            Lambda::call(0, vec![Lambda::val(0)]),
            Some(Lambda::def(1, Lambda::val(1), None)),
        );
        let reduced = full_reduce(lambda);
        assert_eq!(reduced, Lambda::def(1, Lambda::val(1), None))
    }
}
