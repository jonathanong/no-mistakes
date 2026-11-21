pub fn redirect_targets(line: &str) -> Vec<String> {
    output_redirects(line)
        .into_iter()
        .map(|hit| hit.target)
        .collect()
}

pub fn mask_output_redirects(line: &str) -> String {
    let mut masked = line.to_string();
    for hit in output_redirects(line) {
        if hit.end <= masked.len() {
            masked.replace_range(hit.start..hit.end, &" ".repeat(hit.end - hit.start));
        }
    }
    masked
}

struct Hit {
    target: String,
    start: usize,
    end: usize,
}

fn output_redirects(line: &str) -> Vec<Hit> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut hits = Vec::new();
    let mut quote = None;
    let mut arith = 0i32;
    let mut index = 0;
    while index < chars.len() {
        let (_, ch) = chars[index];
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if super::starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && super::starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if arith > 0 || super::starts(&chars, index, &['<', '<']) {
            index += if super::starts(&chars, index, &['<', '<']) {
                2
            } else {
                1
            };
            continue;
        }
        if ch == '>' {
            if let Some(hit) = filename_redirect(&chars, line, &mut index) {
                hits.push(hit);
            }
            continue;
        }
        index += 1;
    }
    hits
}

fn filename_redirect(chars: &[(usize, char)], line: &str, index: &mut usize) -> Option<Hit> {
    let double = super::next_is(chars, *index, '>');
    let amp = !double && super::next_is(chars, *index, '&');
    if !double && !amp && super::next_is(chars, *index, '=') {
        *index += 1;
        return None;
    }
    let mut at = *index + usize::from(double) + usize::from(amp) + 1;
    while chars.get(at).is_some_and(|(_, ch)| ch.is_whitespace()) {
        at += 1;
    }
    if amp && bare_fd(line, chars, at) {
        *index = at;
        return None;
    }
    let Some((start, ch)) = chars.get(at).copied() else {
        *index = at;
        return None;
    };
    let (target, next_at) = if ch == '"' || ch == '\'' {
        quoted_target(line, start, ch)
    } else {
        bare_target(line, start)
    };
    *index = next_at;
    (!target.is_empty()).then_some(Hit {
        target,
        start,
        end: next_at,
    })
}

fn bare_fd(line: &str, chars: &[(usize, char)], at: usize) -> bool {
    let Some((start, ch)) = chars.get(at).copied() else {
        return false;
    };
    if ch == '"' || ch == '\'' {
        return false;
    }
    let rest = &line[start..];
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let token = rest[..end].trim_end_matches([';', '&']);
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
}

fn quoted_target(line: &str, start: usize, quote: char) -> (String, usize) {
    let body_at = start + quote.len_utf8();
    let body = &line[body_at..];
    match body.find(quote) {
        Some(end) => {
            let text = &body[..end];
            let kept = if quote == '\'' && whole_var(text) {
                ""
            } else {
                text
            };
            (kept.to_string(), body_at + end + quote.len_utf8())
        }
        None => (String::new(), line.len()),
    }
}

fn whole_var(text: &str) -> bool {
    let Some(rest) = text.strip_prefix('$') else {
        return false;
    };
    match rest.strip_prefix('{') {
        Some(body) => body.strip_suffix('}').is_some_and(shell_name),
        None => shell_name(rest),
    }
}

fn shell_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn bare_target(line: &str, start: usize) -> (String, usize) {
    let rest = &line[start..];
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let token = rest[..end].trim_end_matches([';', '&']);
    (token.to_string(), start + end)
}
