use std::{collections::VecDeque, fmt::Display, time::Instant};

use compiler::compile;
use helpers::{format_lambda, format_lambda_indented};
use parser::{parse_program, Binder};
use reducer::full_reduce;

mod compiler;
mod helpers;
mod parser;
mod reducer;

// make this copy-able
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lambda {
    Variable(usize),
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
            Lambda::Variable(value) => write!(f, "{value}"),
            Lambda::Definition {
                name_index: input,
                body,
                parameter,
            } => {
                write!(f, "{input}(")?;
                write!(f, "{}", *body)?;
                write!(f, ")")?;

                if let Some(value) = parameter {
                    write!(f, ".({value})")?;
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
    pub(crate) fn new_var(value: &str, binder: &mut Binder) -> Self {
        let index = binder.find_index(value);
        let index = if let Some(index) = index {
            index
        } else {
            binder.new_binding(value.to_owned())
        };
        // println!("got index {index} for binding {value}");
        Lambda::Variable(index)
    }
    pub(crate) fn new_call(
        function_name: &str,
        parameter: Vec<Lambda>,
        binder: &mut Binder,
    ) -> Self {
        let name_index = binder
            .find_index(function_name)
            .unwrap_or_else(|| panic!("Unknown function name: {function_name}"));
        Lambda::Call {
            name_index,
            parameters: VecDeque::from(parameter),
        }
    }

    pub(crate) fn var(name_index: usize) -> Self {
        Lambda::Variable(name_index)
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

fn run_program(text: &str) -> (Lambda, Vec<String>) {
    let compiled = compile(text);
    println!("{compiled}");
    let (lambda, bindings) = parse_program(&compiled);
    let bindings_clone = bindings.clone();
    let print =
        |lambda: &Lambda| println!("{}", format_lambda_indented(lambda, &bindings, 0, true));
    // println!("parsed {}", format_lambda(&lambda, &bindings));
    println!("{bindings_clone:?}");
    (full_reduce(lambda, 50), bindings_clone)
}

fn calculate_5_times_6() {
    let text = "
    let zero f,x(x);
    let succ n,f,x(
            f.(n.f.x)
    );
    let mul n,m(
        f,x(m.(n.f).x)
    );
    let m succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.zero))))))))));
    let n succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.(succ.zero)))))))));
    mul.m.n
    ";
    let now = Instant::now();
    let compiled = compile(text);
    println!("compiled in {}ms", now.elapsed().as_micros());
    let now = Instant::now();
    let (lambda, bindings) = parse_program(&compiled);
    println!("parsed in {}ms", now.elapsed().as_micros());
    let now = Instant::now();
    let result = full_reduce(lambda, 1000);
    println!("run in {}ms", now.elapsed().as_micros());

    // println!("{}", format_lambda(&result, &bindings));
}

fn main() {
    calculate_5_times_6();
}

#[cfg(test)]
mod tests {
    use crate::{helpers::format_lambda, run_program, Lambda};

    #[test]
    fn simple_reduction() {
        let text = "f(f.y).x(x)";
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(2));
    }

    #[test]
    fn not_true() {
        let text = "true(not(not.true).b(b.f.t)).c(d(c))";
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(4));
    }

    #[test]
    fn not_false() {
        let text = "false(not(not.false).b(b.f.t)).c(d(d))";
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(5));
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
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(7));
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
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(7));
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
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(7));
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
        let (reduced, _bindings) = run_program(text);
        assert_eq!(reduced, Lambda::var(6));
    }

    #[test]
    fn church_numerals() {
        let text = "
        let zero f,x(x);
        let succ n,f,x(
            f.(n.f.x)
        );
        succ.(succ.zero)
        ";
        let (result, bindings) = run_program(text);
        assert_eq!(
            result,
            Lambda::def(
                4,
                Lambda::def(
                    5,
                    Lambda::call(4, vec![Lambda::call(4, vec![Lambda::var(5)])]),
                    None
                ),
                None
            )
        );
        println!("{}", format_lambda(&result, &bindings));
    }

    #[test]
    fn adding() {
        let text = "
        let add m,n(
            f,x(
                (m.f).(n.f.x)
            )
        );
        let succ n,f,x(
                f.(n.f.x)
        );
        let zero f,x(x);
        let m succ.(succ.(succ.zero));
        let n succ.(succ.zero);
        add.m.n
        ";

        let (result, bindings) = run_program(text);
        assert_eq!(
            format_lambda(&result, &bindings),
            "f(x(f.(f.(f.(f.(f.(x)))))))"
        );
    }

    #[test]
    fn multiplying() {
        let text = "
        let zero f,x(x);
        let succ n,f,x(
                f.(n.f.x)
        );
        let mul n,m(
            f,x(m.(n.f).x)
        );
        let m succ.(succ.(succ.zero));
        let n succ.(succ.zero);
        mul.m.n
        ";

        let (result, bindings) = run_program(text);
        assert_eq!(
            format_lambda(&result, &bindings),
            "f(x(f.(f.(f.(f.(f.(f.(x))))))))"
        );
    }

    #[test]
    #[should_panic]
    fn omega() {
        let text = "
        let omega x(x.x);
        omega.omega
        ";

        let (_result, _bindings) = run_program(text);
    }

    fn pred() {}
    fn fiboncacci() {}
}
