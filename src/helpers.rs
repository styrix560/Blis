use crate::Lambda;

pub(crate) fn find_block_end(text: &str) -> Option<usize> {
    let mut stack = 0;
    for (index, char) in text.char_indices() {
        if char == '(' {
            stack += 1;
        }
        if char == ')' {
            stack -= 1;
            assert!(stack >= 0);
            if stack == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(crate) fn format_lambda_indented(
    lambda: &Lambda,
    bindings: &Vec<String>,
    depth: usize,
    is_on_newline: bool,
) -> String {
    let whitespaces = " ".repeat(depth);
    let mut string = String::new();
    match lambda {
        Lambda::Variable(value) => string += &bindings[*value],
        Lambda::Definition {
            name_index: input,
            body,
            parameter,
        } => {
            string += &format!(
                "{}{}({}\n{})",
                if is_on_newline {
                    whitespaces.clone()
                } else {
                    "".to_string()
                },
                &bindings[*input],
                whitespaces,
                format_lambda_indented(body, bindings, depth + 1, true)
            );

            if let Some(value) = parameter {
                string += &format!(
                    "\n{whitespaces}.{}",
                    format_lambda_indented(value, bindings, depth + 1, false)
                );
            };
        }
        Lambda::Call {
            name_index: input,
            parameters: args,
        } => {
            string += (if is_on_newline {
                whitespaces.clone()
            } else {
                "".to_string()
            } + &bindings[*input])
                .as_str();
            for arg in args {
                string += &format!(
                    "\n{}.({})",
                    whitespaces.clone(),
                    format_lambda_indented(arg, bindings, depth + 1, false)
                );
            }
        }
    }
    string
}

pub(crate) fn format_lambda(lambda: &Lambda, bindings: &Vec<String>) -> String {
    let mut string = String::new();
    match lambda {
        Lambda::Variable(value) => string += &bindings[*value],
        Lambda::Definition {
            name_index: input,
            body,
            parameter,
        } => {
            string += &format!("{}({})", &bindings[*input], format_lambda(body, bindings));

            if let Some(value) = parameter {
                string += &format!(".{}", format_lambda(value, bindings));
            };
        }
        Lambda::Call {
            name_index: input,
            parameters: args,
        } => {
            string += &bindings[*input];
            for arg in args {
                string += &format!(".({})", format_lambda(arg, bindings));
            }
        }
    }
    string
}
