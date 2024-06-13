use std::{
    collections::VecDeque,
    fmt::{write, Display},
};

mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Lambda {
    Value(String),
    Definition {
        input: String,
        body: Box<Lambda>,
        parameter: Option<Box<Lambda>>,
    },
    Call {
        input: String,
        args: VecDeque<Lambda>,
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
            Lambda::Call { input, args } => {
                write!(f, "{}", input)?;
                for arg in args {
                    write!(f, ".{}", arg)?;
                }
                Ok(())
            }
        }
    }
}

impl Lambda {
    fn val(value: &str) -> Self {
        Lambda::Value(value.to_string())
    }
    fn call(input: &str, replacement: VecDeque<Lambda>) -> Self {
        Lambda::Call {
            input: input.to_string(),
            args: replacement,
        }
    }
    fn def(input: &str, body: Lambda, parameter: Option<Lambda>) -> Self {
        Lambda::Definition {
            input: input.to_string(),
            body: Box::new(body),
            parameter: parameter.map(Box::new),
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {}
