use std::collections::VecDeque;

use crate::Lambda;

fn insert_arguments(root: &mut Lambda, args: &mut VecDeque<Lambda>) {
    if args.is_empty() {
        return;
    }
    match root {
        Lambda::Variable(name) => {
            let args = args.drain(..);
            *root = Lambda::call(*name, args.collect());
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

fn replace(name: usize, replacement: &Lambda, body: &Lambda) -> Lambda {
    // println!("reducing {body} with {name} -> {replacement}");
    match body {
        Lambda::Variable(value) => {
            if value == &name {
                replacement.clone()
            } else {
                Lambda::var(*value)
            }
        }
        Lambda::Definition {
            name_index,
            body,
            parameter,
        } => {
            if name == *name_index {
                return body.as_ref().clone();
            }
            let new_body = replace(name, replacement, body.as_ref());

            let new_parameter = parameter
                .as_ref()
                .map(|p| replace(name, replacement, p.as_ref()))
                .map(Box::new);

            Lambda::Definition {
                name_index: *name_index,
                body: Box::new(new_body),
                parameter: new_parameter,
            }
        }
        Lambda::Call {
            name_index,
            parameters,
        } => {
            let mut new_parameter: VecDeque<Lambda> = parameters
                .iter()
                .map(|p| replace(name, replacement, p))
                .collect();

            if *name_index == name {
                // println!("{new_parameter:?}");
                let mut replacement = replacement.clone();
                insert_arguments(&mut replacement, &mut new_parameter);
                replacement
            } else {
                Lambda::call(*name_index, new_parameter.into_iter().collect())
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
        let parameter = Box::new(full_reduce(*parameter, 10));

        return replace(name_index, &parameter, &body);
    }
    unreachable!()
}

fn find_reducible(root: Lambda) -> Result<Lambda, Lambda> {
    match root {
        Lambda::Variable(_) => Err(root),
        Lambda::Definition {
            name_index,
            body,
            parameter,
        } => {
            let new_body = find_reducible(*body.clone());
            if let Ok(new_body) = new_body {
                Ok(Lambda::def(name_index, new_body, parameter.map(|p| *p)))
            } else if parameter.is_some() {
                Ok(reduce(Lambda::def(
                    name_index,
                    *body,
                    parameter.map(|p| *p),
                )))
            } else {
                Err(Lambda::def(
                    name_index,
                    new_body.unwrap_err(),
                    parameter.map(|p| *p),
                ))
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

pub(crate) fn full_reduce(mut root: Lambda, iterations: usize) -> Lambda {
    for iteration in 0..iterations {
        let result = find_reducible(root);
        if let Err(result) = result {
            return result;
        }
        root = result.unwrap();
    }
    panic!("Term was not reducible in {iterations} iterations");
}

pub(crate) fn full_reduce_debug<F>(mut root: Lambda, print: F) -> Lambda
where
    F: FnOnce(&Lambda) + std::marker::Copy,
{
    for iteration in 0..50 {
        print(&root);
        let result: Result<Lambda, Lambda> = find_reducible(root);
        if let Err(result) = result {
            // println!("done in {iteration} iterations");
            return result;
        }
        root = result.unwrap();
    }
    panic!();
}

#[cfg(test)]
mod tests {

    use crate::{reducer::full_reduce, Lambda};

    #[test]
    fn no_reduction() {
        // a(a)
        let lambda = Lambda::def(0, Lambda::var(0), None);
        let reduced = full_reduce(lambda.clone(), 50);
        assert_eq!(lambda, reduced);
    }

    #[test]
    fn simple_reduction() {
        let lambda = Lambda::def(
            0,
            Lambda::call(0, vec![Lambda::var(1)]),
            Some(Lambda::def(2, Lambda::var(2), None)),
        );
        let reduced = full_reduce(lambda, 50);
        assert_eq!(reduced, Lambda::var(1))
    }

    #[test]
    fn complex_reduction() {
        // a(b(a.b)).(c(d(d)).5).3
        let lambda = Lambda::def(
            0,
            Lambda::def(
                1,
                Lambda::call(0, vec![Lambda::var(1)]),
                Some(Lambda::var(2)),
            ),
            Some(Lambda::def(
                3,
                Lambda::def(4, Lambda::var(4), None),
                Some(Lambda::var(5)),
            )),
        );
        let reduced = full_reduce(lambda, 50);
        assert_eq!(reduced, Lambda::var(2));
    }

    #[test]
    fn name_collision() {
        // 0(2(0.2)).1(1) => 2(1(1).2) => 2(2)
        let lambda = Lambda::def(
            0,
            Lambda::def(2, Lambda::call(0, vec![Lambda::var(2)]), None),
            Some(Lambda::def(1, Lambda::var(1), None)),
        );
        let reduced = full_reduce(lambda, 50);
        assert_eq!(reduced, Lambda::def(2, Lambda::var(2), None))
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
                        vec![Lambda::call(1, vec![Lambda::call(0, vec![Lambda::var(3)])])],
                    ),
                    Some(Lambda::def(6, Lambda::var(6), None)),
                ),
                Some(Lambda::def(5, Lambda::var(5), None)),
            ),
            Some(Lambda::def(4, Lambda::var(4), None)),
        );
        let reduced = full_reduce(lambda, 50);
        assert_eq!(reduced, Lambda::var(3))
    }

    #[test]
    fn calling_with_itself() {
        let lambda = Lambda::def(
            0,
            Lambda::call(0, vec![Lambda::var(0)]),
            Some(Lambda::def(1, Lambda::var(1), None)),
        );
        let reduced = full_reduce(lambda, 50);
        assert_eq!(reduced, Lambda::def(1, Lambda::var(1), None))
    }

    // #[test]
    // fn nested_within_itself() {
    //     // f(f(f).a(a)).f(f.5) => f(f.5)(f(f.5)).a(a)
    //     // f a 5
    //     let lambda = Lambda::def(
    //         0,
    //         Lambda::def(
    //             0,
    //             Lambda::var(0),
    //             Some(Lambda::def(1, Lambda::var(1), None)),
    //         ),
    //         Some(Lambda::def(0, Lambda::call(0, vec![Lambda::var(2)]), None)),
    //     );
    //     let reduced
    // }
}
