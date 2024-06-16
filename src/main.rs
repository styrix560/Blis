use std::{collections::VecDeque, fmt::Display};

use compiler::compile;
use parser::{parse_program, Binder};
use reducer::full_reduce;

mod compiler;
mod parser;
mod reducer;

// make this copy-able
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lambda {
    Value(usize),
    Definition {
        name_index: usize,
        body: Box<Lambda>,
        parameter: Option<Box<Lambda>>,
    },
    Call {
        name_index: usize,
        parameters: VecDeque<Lambda>,
    },
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Value(value) => write!(f, "{value}"),
            Lambda::Definition {
                name_index: input,
                body,
                parameter,
            } => {
                write!(f, "{input}(")?;
                write!(f, "{}", *body)?;
                write!(f, ")")?;

                if let Some(value) = parameter {
                    write!(f, ".{value}")?;
                };
                Ok(())
            }
            Lambda::Call {
                name_index: input,
                parameters: args,
            } => {
                write!(f, "{input}")?;
                for arg in args {
                    write!(f, ".({arg})")?;
                }
                Ok(())
            }
        }
    }
}

impl Lambda {
    pub(crate) fn new_val(value: &str, binder: &mut Binder) -> Self {
        let index = binder.find_index(value);
        let index = if let Some(index) = index {
            index
        } else {
            binder.new_binding(value.to_owned())
        };
        println!("got index {index} for binding {value}");
        Lambda::Value(index)
    }
    pub(crate) fn new_call(
        function_name: &str,
        parameter: Vec<Lambda>,
        binder: &mut Binder,
    ) -> Self {
        let name_index = binder
            .find_index(function_name)
            .expect("Unknown function name");
        Lambda::Call {
            name_index,
            parameters: VecDeque::from(parameter),
        }
    }

    pub(crate) fn val(name_index: usize) -> Self {
        Lambda::Value(name_index)
    }

    pub(crate) fn call(name_index: usize, parameters: Vec<Lambda>) -> Self {
        Lambda::Call {
            name_index,
            parameters: parameters.into_iter().collect(),
        }
    }
    pub(crate) fn def(name_index: usize, body: Lambda, parameter: Option<Lambda>) -> Self {
        Lambda::Definition {
            name_index,
            body: Box::new(body),
            parameter: parameter.map(Box::new),
        }
    }
}

fn run_program(text: &str) -> Lambda {
    let compiled = compile(text);
    let (lambda, bindings) = parse_program(&compiled);
    println!("{bindings:?}");
    full_reduce(lambda)
}

fn main() {
    println!("{}", run_program("f(f.y).x(x)"));
}

#[cfg(test)]
mod tests {
    use crate::{run_program, Lambda};

    #[test]
    fn simple_reduction() {
        let text = "f(f.y).x(x)";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(2));
    }

    #[test]
    fn not_true() {
        let text = "true(not(not.true).b(b.f.t)).c(d(c))";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(4));
    }

    #[test]
    fn not_false() {
        let text = "false(not(not.false).b(b.f.t)).c(d(d))";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(5));
    }

    #[test]
    fn and_false_false() {
        let text = "
        false(
            true(
                e(
                    g(
                        e.g.f
                    )
                ).false.(false.t.f)
            ).c(
                d(
                    c
                )
            )
        ).a(
            b(
                b
            )
        )
        ";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(7));
    }
    #[test]
    fn and_true_false() {
        let text = "
        false(
            true(
                e(
                    g(
                        e.g.f
                    )
                ).true.(false.t.f)
            ).c(
                d(
                    c
                )
            )
        ).a(
            b(
                b
            )
        )
        ";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(7));
    }
    #[test]
    fn and_false_true() {
        let text = "
        false(
            true(
                e(
                    g(
                        e.g.f
                    )
                ).false.(true.t.f)
            ).c(
                d(
                    c
                )
            )
        ).a(
            b(
                b
            )
        )
        ";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(7));
    }

    #[test]
    fn and_true_true() {
        let text = "
        false(
            true(
                e(
                    g(
                        e.g.f
                    )
                ).true.(true.t.f)
            ).c(
                d(
                    c
                )
            )
        ).a(
            b(
                b
            )
        )
        ";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val(6));
    }

    #[test]
    fn church_numerals() {
        let text = "
        zero(
            succ(
                f(
                    succ.(succ.zero)
                ).a(a)
            ).n(
                f(
                    x(
                        f.(n.f.x)
                    )
                )
            )
        ).f(
            x(
                x
            )
        )
        ";
        let result = run_program(text);
        assert_eq!(
            result,
            Lambda::def(
                4,
                Lambda::def(
                    5,
                    Lambda::call(4, vec![Lambda::call(4, vec![Lambda::val(5)])]),
                    None
                ),
                None
            )
        );
    }
}
