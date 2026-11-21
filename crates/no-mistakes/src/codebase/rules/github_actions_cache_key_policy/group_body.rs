use super::super::super::shell_cmd::operator_width;

pub(super) fn inheriting(body: &str) -> Vec<(&str, bool)> {
    let chars: Vec<(usize, char)> = body.char_indices().collect();
    let mut out = Vec::new();
    let mut start = 0;
    let mut inherit = true;
    let mut quote = None;
    let mut arith = 0i32;
    let mut index = 0;
    while index < chars.len() {
        let (at, ch) = chars[index];
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if arith > 0 {
            index += 1;
            continue;
        }
        let width = operator_width(&chars, index);
        if width == 0 {
            index += 1;
            continue;
        }
        let piped = ch == '|' && width == 1;
        push_part(&mut out, body, start, at, inherit && !piped);
        inherit = true;
        index += width;
        start = chars
            .get(index)
            .map(|(next, _)| *next)
            .unwrap_or(body.len());
    }
    push_part(&mut out, body, start, body.len(), inherit);
    out
}

fn push_part<'a>(
    out: &mut Vec<(&'a str, bool)>,
    body: &'a str,
    start: usize,
    end: usize,
    inherit: bool,
) {
    let part = body[start..end].trim();
    if !part.is_empty() {
        out.push((part, inherit));
    }
}

pub(super) fn starts(chars: &[(usize, char)], index: usize, prefix: &[char]) -> bool {
    prefix.iter().enumerate().all(|(offset, expected)| {
        chars
            .get(index + offset)
            .is_some_and(|(_, ch)| ch == expected)
    })
}

pub(super) fn closed_group(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let mut chars = line.chars();
    let opener = chars.next()?;
    let closer = match opener {
        '{' if brace_word(line) => '}',
        '(' => ')',
        _ => return None,
    };
    let mut braces = i32::from(opener == '{');
    let mut parens = i32::from(opener == '(');
    let mut quote = None;
    let mut escape = false;
    let mut end = None;
    for (index, ch) in line.char_indices().skip(1) {
        if let Some(open) = quote {
            if escape {
                escape = false;
            } else if ch == '\\' && open == '"' {
                escape = true;
            } else if ch == open {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            escape = false;
            continue;
        }
        match ch {
            '{' => braces += 1,
            '}' if braces > 0 => {
                braces -= 1;
                if braces == 0 && parens == 0 && closer == '}' {
                    end = Some(index);
                    break;
                }
            }
            '(' => parens += 1,
            ')' if parens > 0 => {
                parens -= 1;
                if parens == 0 && braces == 0 && closer == ')' {
                    end = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    Some((
        line[opener.len_utf8()..end].trim(),
        line[end + closer.len_utf8()..].trim(),
    ))
}

fn brace_word(line: &str) -> bool {
    let rest = &line['{'.len_utf8()..];
    rest.is_empty()
        || rest.starts_with(char::is_whitespace)
        || rest.starts_with(['}', '>', '<', ';', '&', '|', '#'])
}
