use std::{
    collections::VecDeque,
    fmt::{write, Display},
};

use parser::parse_program;
use reducer::full_reduce;

mod parser;
mod reducer;

// make this copy-able
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lambda {
    Value(String),
    Definition {
        input: String,
        body: Box<Lambda>,
        parameter: Option<Box<Lambda>>,
    },
    Call {
        function_name: String,
        parameters: VecDeque<Lambda>,
    },
}

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Value(value) => write!(f, "{value}"),
            Lambda::Definition {
                input,
                body,
                parameter,
            } => {
                write!(f, "{input}(")?;
                write!(f, "{}", *body)?;
                write!(f, ")")?;

                if let Some(value) = parameter {
                    write!(f, ".{}", value)?
                };
                Ok(())
            }
            Lambda::Call {
                function_name: input,
                parameters: args,
            } => {
                write!(f, "{}", input)?;
                for arg in args {
                    write!(f, ".({})", arg)?;
                }
                Ok(())
            }
        }
    }
}

impl Lambda {
    pub(crate) fn val(value: &str) -> Self {
        Lambda::Value(value.to_string())
    }
    pub(crate) fn call(function_name: &str, parameter: Vec<Lambda>) -> Self {
        Lambda::Call {
            function_name: function_name.to_string(),
            parameters: VecDeque::from(parameter),
        }
    }
    pub(crate) fn def(input: &str, body: Lambda, parameter: Option<Lambda>) -> Self {
        Lambda::Definition {
            input: input.to_string(),
            body: Box::new(body),
            parameter: parameter.map(Box::new),
        }
    }
}

fn run_program(text: &str) -> Lambda {
    let lambda = parse_program(text);
    full_reduce(lambda)
}

fn main() {
    println!("{:?}", parser::parse_program("f(f.y).x(x)"))
}

#[cfg(test)]
mod tests {
    use crate::{run_program, Lambda};

    #[test]
    fn simple_reduction() {
        let text = "f(f.y).x(x)";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val("y"));
    }

    #[test]
    fn not_true() {
        let text = "true(not(not.true).b(b.f.t)).c(d(c))";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val("f"));
    }

    #[test]
    fn not_false() {
        let text = "false(not(not.false).b(b.f.t)).c(d(d))";
        let reduced = run_program(text);
        assert_eq!(reduced, Lambda::val("t"));
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
        assert_eq!(reduced, Lambda::val("f"));
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
        assert_eq!(reduced, Lambda::val("f"));
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
        assert_eq!(reduced, Lambda::val("f"));
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
        assert_eq!(reduced, Lambda::val("t"));
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
                "f11",
                Lambda::def(
                    "x11",
                    Lambda::call("f11", vec![Lambda::call("f11", vec![Lambda::val("x11")])]),
                    None
                ),
                None
            )
        );
    }
}
