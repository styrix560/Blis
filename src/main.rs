use std::{
    collections::VecDeque,
    fmt::{write, Display},
};

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
        parameter: VecDeque<Lambda>,
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
                parameter: args,
            } => {
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
    pub(crate) fn val(value: &str) -> Self {
        Lambda::Value(value.to_string())
    }
    pub(crate) fn call(function_name: &str, parameter: VecDeque<Lambda>) -> Self {
        Lambda::Call {
            function_name: function_name.to_string(),
            parameter,
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

fn main() {
    println!("{:?}", parser::parse_program("f(f.y).x(x)"))
}

#[cfg(test)]
mod tests {}
