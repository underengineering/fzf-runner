use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum Arg {
    Text(String),
    Field(char),
}

fn parse_escaped_char(chars: &mut Peekable<Chars>) -> Option<char> {
    if let Some(ch) = chars.next() {
        match ch {
            '\\' if matches!(chars.peek(), Some('"' | '`' | '$' | '\\')) => {
                Some(chars.next().unwrap())
            }
            ch => Some(ch),
        }
    } else {
        None
    }
}

fn parse_quoted_argument(chars: &mut Peekable<Chars>) -> String {
    let mut current_arg = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => break,
            '\\' if chars.peek() == Some(&'\\') => {
                chars.next().unwrap();

                // NOTE: Discard invalid arguments
                if let Some(arg) = parse_escaped_char(chars) {
                    current_arg.push(arg);
                }
            }
            ch => current_arg.push(ch),
        }
    }

    current_arg
}

/// As defined in
/// <https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#exec-variables>
pub fn parse_arguments(text: &str) -> Vec<Arg> {
    let mut args = Vec::new();
    let mut current_arg = String::new();

    let mut chars = text.trim_start().chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            ' ' => {
                if chars.peek() != Some(&' ') && !current_arg.is_empty() {
                    args.push(Arg::Text(std::mem::take(&mut current_arg)));
                }
            }
            '"' => args.push(Arg::Text(parse_quoted_argument(&mut chars))),
            '%' => {
                args.push(Arg::Field(chars.next().unwrap()));
            }
            ch => current_arg.push(ch),
        }
    }

    if !current_arg.is_empty() {
        args.push(Arg::Text(current_arg));
    }

    args
}

#[cfg(test)]
mod tests {
    use super::{parse_arguments, Arg};

    #[test]
    fn parsing_works() {
        let result = parse_arguments("foo    bar baz");
        assert_eq!(
            result,
            vec![
                Arg::Text("foo".to_string()),
                Arg::Text("bar".to_string()),
                Arg::Text("baz".to_string())
            ]
        );
    }

    #[test]
    fn parsing_field_works() {
        let result = parse_arguments("foo %U bar");
        assert_eq!(
            result,
            vec![
                Arg::Text("foo".to_string()),
                Arg::Field('U'),
                Arg::Text("bar".to_string())
            ]
        );
    }

    #[test]
    fn parsing_escaped_works() {
        let result = parse_arguments("foo  \"\\\\$bar \\\\\\\\ \\\\\"baz\" qux");
        assert_eq!(
            result,
            vec![
                Arg::Text("foo".to_string()),
                Arg::Text("$bar \\ \"baz".to_string()),
                Arg::Text("qux".to_string())
            ]
        );
    }

    #[test]
    fn parsing_unbalanced_quote_works() {
        let result = parse_arguments("foo \"\\\\");
        assert_eq!(
            result,
            vec![Arg::Text("foo".to_string()), Arg::Text("".to_string())]
        );
    }

    #[test]
    fn parsing_unbalanced_quote_works_2() {
        let result = parse_arguments("foo \"\\\\ bar");
        assert_eq!(
            result,
            vec![Arg::Text("foo".to_string()), Arg::Text(" bar".to_string())]
        );
    }
}
