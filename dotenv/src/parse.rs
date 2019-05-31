use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::errors::*;

// for readability's sake
pub type ParsedLine = Result<Option<(String, String)>>;

pub fn parse_line(
    line: &str,
    mut substitution_data: &mut HashMap<String, Option<String>>,
) -> ParsedLine {
    lazy_static! {
        static ref LINE_REGEX: Regex = Regex::new(
            r#"(?x)
        ^(
          \s*
          (
            \#.*|                           # A comment, or...
            \s*|                            # ...an empty string, or...
            (export\s+)?                    # ...(optionally preceded by "export")...
            (?P<key>[A-Za-z_][A-Za-z0-9_]*) # ...a key,...
            =                               # ...then an equal sign,...
            (?P<value>.+?)?                 # ...and then its corresponding value.
          )\s*
        )
        [\r\n]*
        $
      "#
        )
        .unwrap();
    }

    LINE_REGEX.captures(line).map_or(
        Err(Error::LineParse(ParseError::new(line.into(), 0))),
        |captures| {
            let key = named_string(&captures, "key");
            let value = named_string(&captures, "value");

            match (key, value) {
                (Some(k), Some(v)) => {
                    let parsed_value = parse_value(&v, &mut substitution_data)?;
                    substitution_data.insert(k.to_owned(), Some(parsed_value.to_owned()));

                    Ok(Some((k, parsed_value)))
                }
                (Some(k), None) => {
                    substitution_data.insert(k.to_owned(), None);
                    // Empty string for value.
                    Ok(Some((k, String::from(""))))
                }
                _ => {
                    // If there's no key, but capturing did not
                    // fail, we're dealing with a comment
                    Ok(None)
                }
            }
        },
    )
}

fn named_string(captures: &Captures<'_>, name: &str) -> Option<String> {
    captures
        .name(name)
        .and_then(|v| Some(v.as_str().to_owned()))
}

#[derive(Eq, PartialEq)]
enum SubstitutionMode {
    None,
    Block,
    EscapedBlock,
}

fn parse_value(
    input: &str,
    substitution_data: &mut HashMap<String, Option<String>>,
) -> Result<String> {
    let mut strong_quote = false; // '
    let mut weak_quote = false; // "
    let mut escaped = false;
    let mut expecting_end = false;

    //FIXME can this be done without yet another allocation per line?
    let mut output = String::new();

    let mut substitution_mode = SubstitutionMode::None;
    let mut substitution_name = String::new();

    for (index, c) in input.chars().enumerate() {
        //the regex _should_ already trim whitespace off the end
        //expecting_end is meant to permit: k=v #comment
        //without affecting: k=v#comment
        //and throwing on: k=v w
        if expecting_end {
            if c == ' ' || c == '\t' {
                continue;
            } else if c == '#' {
                break;
            } else {
                return Err(Error::LineParse(ParseError::new(input.to_owned(), index)));
            }
        } else if escaped {
            //TODO I tried handling literal \n \r but various issues
            //imo not worth worrying about until there's a use case
            //(actually handling backslash 0x10 would be a whole other matter)
            //then there's \v \f bell hex... etc
            match c {
                '\\' | '\'' | '"' | '$' | ' ' => output.push(c),
                _ => {
                    return Err(Error::LineParse(ParseError::new(input.to_owned(), index)));
                }
            }

            escaped = false;
        } else if strong_quote {
            if c == '\'' {
                strong_quote = false;
            } else {
                output.push(c);
            }
        } else if substitution_mode != SubstitutionMode::None {
            if c.is_alphanumeric() {
                substitution_name.push(c);
            } else {
                match substitution_mode {
                    SubstitutionMode::None => unreachable!(),
                    SubstitutionMode::Block => {
                        if c == '{' && substitution_name.is_empty() {
                            substitution_mode = SubstitutionMode::EscapedBlock;
                        } else {
                            apply_substitution(
                                substitution_data,
                                &substitution_name.drain(..).collect::<String>(),
                                &mut output,
                            );
                            if c == '$' {
                                substitution_mode = if !strong_quote && !escaped {
                                    SubstitutionMode::Block
                                } else {
                                    SubstitutionMode::None
                                }
                            } else {
                                substitution_mode = SubstitutionMode::None;
                                output.push(c);
                            }
                        }
                    }
                    SubstitutionMode::EscapedBlock => {
                        if c == '}' {
                            substitution_mode = SubstitutionMode::None;
                            apply_substitution(
                                substitution_data,
                                &substitution_name.drain(..).collect::<String>(),
                                &mut output,
                            );
                        } else {
                            substitution_name.push(c);
                        }
                    }
                }
            }
        } else if c == '$' {
            substitution_mode = if !strong_quote && !escaped {
                SubstitutionMode::Block
            } else {
                SubstitutionMode::None
            }
        } else if weak_quote {
            if c == '"' {
                weak_quote = false;
            } else if c == '\\' {
                escaped = true;
            } else {
                output.push(c);
            }
        } else if c == '\'' {
            strong_quote = true;
        } else if c == '"' {
            weak_quote = true;
        } else if c == '\\' {
            escaped = true;
        } else if c == ' ' || c == '\t' {
            expecting_end = true;
        } else {
            output.push(c);
        }
    }

    //XXX also fail if escaped? or...
    if substitution_mode == SubstitutionMode::EscapedBlock || strong_quote || weak_quote {
        let value_length = input.len();
        Err(Error::LineParse(ParseError::new(
            input.to_owned(),
            if value_length == 0 {
                0
            } else {
                value_length - 1
            },
        )))
    } else {
        apply_substitution(
            substitution_data,
            &substitution_name.drain(..).collect::<String>(),
            &mut output,
        );
        Ok(output)
    }
}

fn apply_substitution(
    substitution_data: &mut HashMap<String, Option<String>>,
    substitution_name: &str,
    output: &mut String,
) {
    if let Ok(environment_value) = std::env::var(substitution_name) {
        output.push_str(&environment_value);
    } else {
        let stored_value = substitution_data
            .get(substitution_name)
            .unwrap_or(&None)
            .to_owned();
        output.push_str(&stored_value.unwrap_or_else(String::new));
    };
}

#[cfg(test)]
mod test {
    use crate::iter::Iter;

    use super::*;

    #[test]
    fn test_parse_line_env() {
        let actual_iter = Iter::new(
            r#"
KEY=1
KEY2="2"
KEY3='3'
KEY4='fo ur'
KEY5="fi ve"
KEY6=s\ ix
KEY7=
KEY8=     
KEY9=   # foo
export   SHELL_LOVER=1
"#
            .as_bytes(),
        );

        let expected_iter = vec![
            ("KEY", "1"),
            ("KEY2", "2"),
            ("KEY3", "3"),
            ("KEY4", "fo ur"),
            ("KEY5", "fi ve"),
            ("KEY6", "s ix"),
            ("KEY7", ""),
            ("KEY8", ""),
            ("KEY9", ""),
            ("SHELL_LOVER", "1"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()));

        let mut count = 0;
        for (expected, actual) in expected_iter.zip(actual_iter) {
            assert!(actual.is_ok());
            assert_eq!(expected, actual.ok().unwrap());
            count += 1;
        }

        assert_eq!(count, 10);
    }

    #[test]
    fn test_parse_line_comment() {
        let result: Result<Vec<(String, String)>> = Iter::new(
            r#"
# foo=bar
#    "#
                .as_bytes(),
        )
        .collect();
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_line_invalid() {
        let actual_iter = Iter::new(
            r#"
  invalid    
KEY =val
KEY2= val
very bacon = yes indeed
=value"#
                .as_bytes(),
        );

        let mut count = 0;
        for actual in actual_iter {
            assert!(actual.is_err());
            count += 1;
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_parse_value_escapes() {
        let actual_iter = Iter::new(
            r#"
KEY=my\ cool\ value
KEY2=\$sweet
KEY3="awesome stuff \"mang\""
KEY4='sweet $\fgs'\''fds'
KEY5="'\"yay\\"\ "stuff"
KEY6="lol" #well you see when I say lol wh
"#
            .as_bytes(),
        );

        let expected_iter = vec![
            ("KEY", r#"my cool value"#),
            ("KEY2", r#"$sweet"#),
            ("KEY3", r#"awesome stuff "mang""#),
            ("KEY4", r#"sweet $\fgs'fds"#),
            ("KEY5", r#"'"yay\ stuff"#),
            ("KEY6", "lol"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()));

        for (expected, actual) in expected_iter.zip(actual_iter) {
            assert!(actual.is_ok());
            assert_eq!(expected, actual.unwrap());
        }
    }

    #[test]
    fn test_parse_value_escapes_invalid() {
        let actual_iter = Iter::new(
            r#"
KEY=my uncool value
KEY2="why
KEY3='please stop''
KEY4=h\8u
"#
            .as_bytes(),
        );

        for actual in actual_iter {
            assert!(actual.is_err());
        }
    }
}

#[cfg(test)]
mod variable_substitution_tests {
    use crate::iter::Iter;

    fn assert_parsed_string(input_string: &str, expected_parse_result: Vec<(&str, &str)>) {
        let actual_iter = Iter::new(input_string.as_bytes());
        let expected_count = &expected_parse_result.len();

        let expected_iter = expected_parse_result
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()));

        let mut count = 0;
        for (expected, actual) in expected_iter.zip(actual_iter) {
            assert!(actual.is_ok());
            assert_eq!(expected, actual.ok().unwrap());
            count += 1;
        }

        assert_eq!(count, *expected_count);
    }

    #[test]
    fn variable_in_parenthesis_surrounded_by_quotes() {
        assert_parsed_string(
            r#"
            KEY=test
            KEY1="${KEY}"
            "#,
            vec![("KEY", "test"), ("KEY1", "test")],
        );
    }

    #[test]
    fn substitute_undefined_variables_to_empty_string() {
        assert_parsed_string(r#"KEY=">$KEY1<>${KEY2}<""#, vec![("KEY", "><><")]);
    }

    #[test]
    fn do_not_substitute_variables_with_dollar_escaped() {
        assert_parsed_string(
            "KEY=>\\$KEY1<>\\${KEY2}<",
            vec![("KEY", ">$KEY1<>${KEY2}<")],
        );
    }

    #[test]
    fn do_not_substitute_variables_in_weak_quotes_with_dollar_escaped() {
        assert_parsed_string(
            r#"KEY=">\$KEY1<>\${KEY2}<""#,
            vec![("KEY", ">$KEY1<>${KEY2}<")],
        );
    }

    #[test]
    fn do_not_substitute_variables_in_strong_quotes() {
        assert_parsed_string("KEY='>${KEY1}<>$KEY2<'", vec![("KEY", ">${KEY1}<>$KEY2<")]);
    }

    #[test]
    fn same_variable_reused() {
        assert_parsed_string(
            r#"
    KEY=VALUE
    KEY1=$KEY$KEY
    "#,
            vec![("KEY", "VALUE"), ("KEY1", "VALUEVALUE")],
        );
    }

    #[test]
    fn recursive_substitution() {
        assert_parsed_string(
            r#"
            KEY=${KEY1}+KEY_VALUE
            KEY1=${KEY}+KEY1_VALUE
            "#,
            vec![("KEY", "+KEY_VALUE"), ("KEY1", "+KEY_VALUE+KEY1_VALUE")],
        );
    }

    #[test]
    fn variable_without_parenthesis_is_substituted_before_separators() {
        assert_parsed_string(
            r#"
            KEY1=test_user
            KEY1_1=test_user_with_separator
            KEY=">$KEY1_1<>$KEY1}<>$KEY1{<"
            "#,
            vec![
                ("KEY1", "test_user"),
                ("KEY1_1", "test_user_with_separator"),
                ("KEY", ">test_user_1<>test_user}<>test_user{<"),
            ],
        );
    }

    #[test]
    fn substitute_variable_from_env_variable() {
        std::env::set_var("KEY11", "test_user_env");

        assert_parsed_string(r#"KEY=">${KEY11}<""#, vec![("KEY", ">test_user_env<")]);
    }

    #[test]
    fn substitute_variable_env_variable_overrides_dotenv_in_substitution() {
        std::env::set_var("KEY11", "test_user_env");

        assert_parsed_string(
            r#"
    KEY11=test_user
    KEY=">${KEY11}<"
    "#,
            vec![("KEY11", "test_user"), ("KEY", ">test_user_env<")],
        );
    }

    #[test]
    fn consequent_substitutions() {
        assert_parsed_string(
            r#"
    KEY1=test_user
    KEY2=$KEY1_2
    KEY=>${KEY1}<>${KEY2}<
    "#,
            vec![
                ("KEY1", "test_user"),
                ("KEY2", "test_user_2"),
                ("KEY", ">test_user<>test_user_2<"),
            ],
        );
    }

    #[test]
    fn consequent_substitutions_with_one_missing() {
        assert_parsed_string(
            r#"
    KEY2=$KEY1_2
    KEY=>${KEY1}<>${KEY2}<
    "#,
            vec![("KEY2", "_2"), ("KEY", "><>_2<")],
        );
    }
}

#[cfg(test)]
mod error_tests {
    use crate::errors::Error::LineParse;
    use crate::iter::Iter;

    #[test]
    fn should_not_parse_unfinished_substitutions() {
        let wrong_value = ">${KEY{<";

        let parsed_values: Vec<_> = Iter::new(
            format!(
                r#"
    KEY=VALUE
    KEY1={}
    "#,
                wrong_value
            )
            .as_bytes(),
        )
        .collect();

        assert_eq!(parsed_values.len(), 2);

        if let Ok(first_line) = &parsed_values[0] {
            assert_eq!(first_line, &(String::from("KEY"), String::from("VALUE")))
        } else {
            assert!(false, "Expected the first value to be parsed")
        }
        if let Err(LineParse(err)) = &parsed_values[1] {
            assert_eq!(err.0, wrong_value);
            assert_eq!(err.1, wrong_value.len() - 1)
        } else {
            assert!(false, "Expected the second value not to be parsed")
        }
    }

    #[test]
    fn should_not_parse_illegal_format() {
        let wrong_format = r"<><><>";
        let parsed_values: Vec<_> = Iter::new(wrong_format.as_bytes()).collect();

        assert_eq!(parsed_values.len(), 1);
        if let Err(LineParse(err)) = &parsed_values[0] {
            assert_eq!(err.0, wrong_format);
            assert_eq!(err.1, 0)
        } else {
            assert!(false, "Expected the second value not to be parsed")
        }
    }

    #[test]
    fn should_not_parse_illegal_escape() {
        let wrong_escape = r">\f<";
        let parsed_values: Vec<_> =
            Iter::new(format!("VALUE={}", wrong_escape).as_bytes()).collect();

        assert_eq!(parsed_values.len(), 1);
        if let Err(LineParse(err)) = &parsed_values[0] {
            assert_eq!(err.0, wrong_escape);
            assert_eq!(err.1, wrong_escape.find("\\").unwrap() + 1)
        } else {
            assert!(false, "Expected the second value not to be parsed")
        }
    }
}
