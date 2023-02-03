use std::{ops::Range, str::Chars};

pub(crate) struct CapturedVariables {
    pub(crate) vars: Vec<(String, Range<usize>)>,
    pub(crate) pat_len: usize,
}

pub(crate) fn parse_pattern_string(literal: &str, start_idx: usize) -> (CapturedVariables, String) {
    let mut captured_vars = CapturedVariables {
        vars: Vec::new(),
        pat_len: 0,
    };
    let mut pattern_string = String::new();

    let mut chars = literal.chars();

    let mut annoymous_vars = start_idx;

    while let Some(char) = chars.next() {
        match char {
            '[' => {
                let mut var_name = parse_variable(&mut chars);
                if var_name.is_empty() {
                    var_name = format!("_{}", annoymous_vars.to_string());
                    annoymous_vars += 1;
                }

                captured_vars.vars.push((var_name, 0..0));
                captured_vars.vars.last_mut().unwrap().1.start = pattern_string.len();
            }
            '0' | '1' | 'x' => {
                pattern_string.push(char);
            }

            ']' => {
                captured_vars.vars.last_mut().unwrap().1.end = pattern_string.len();
            }
            ' ' | '_' => continue,
            _ => unimplemented!(),
        }
    }

    captured_vars.pat_len = pattern_string.len();

    for range in captured_vars.vars.iter_mut().map(|(_, range)| range) {
        let end = range.end;
        range.end = captured_vars.pat_len - range.start;
        range.start = captured_vars.pat_len - end;
    }

    (captured_vars, pattern_string)
}

pub(crate) fn parse_variable(chars: &mut Chars) -> String {
    let mut var_name = String::new();

    let mut cloned = chars.clone();

    while let Some(char) = cloned.next() {
        if char.is_ascii_alphanumeric() || char == '_' {
            var_name.push(char);
        } else if char == ':' {
            *chars = cloned;
            break;
        } else if char == ']' {
            return String::new();
        }
    }

    var_name
}
