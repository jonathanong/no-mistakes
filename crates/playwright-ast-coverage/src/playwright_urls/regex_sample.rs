use super::callee::is_candidate_url;

pub fn regex_path_sample(pattern: &str) -> Option<String> {
    let pattern = pattern.trim_start_matches('^').replace(r"\/", "/");
    let mut chars = pattern.chars().peekable();
    let mut sample = String::new();
    let mut started = pattern.starts_with("http://") || pattern.starts_with("https://");
    let mut unsupported = false;
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let Some(next) = chars.next() else {
                break;
            };
            if started && is_literal_path_char(next) && !next.is_ascii_alphanumeric() {
                sample.push(next);
            } else if started {
                unsupported = true;
                break;
            }
            continue;
        }

        if !started {
            if ch == '/' {
                started = true;
                sample.push(ch);
            }
            continue;
        }

        match ch {
            '[' => {
                consume_regex_char_class(&mut chars);
                sample.push('x');
                consume_regex_quantifier(&mut chars);
            }
            '.' => {
                if sample_is_absolute_url_host(&sample) {
                    sample.push('.');
                } else {
                    sample.push('x');
                }
                consume_regex_quantifier(&mut chars);
            }
            '$' => break,
            '|' | '(' | ')' => {
                unsupported = true;
                break;
            }
            ch if is_literal_path_char(ch) => sample.push(ch),
            _ => {
                unsupported = true;
                break;
            }
        }
    }

    if !unsupported && is_candidate_url(&sample) {
        Some(sample)
    } else {
        None
    }
}

pub fn consume_regex_char_class(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    let mut escaped = false;
    for next in chars.by_ref() {
        if escaped {
            escaped = false;
            continue;
        }
        if next == '\\' {
            escaped = true;
            continue;
        }
        if next == ']' {
            break;
        }
    }
}

pub fn consume_regex_quantifier(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while matches!(chars.peek(), Some('+' | '*' | '?' | '{')) {
        let quantifier = chars.next();
        if quantifier == Some('{') {
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
            }
        }
    }
}

pub fn is_literal_path_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~' | '%' | ':')
}

pub fn sample_is_absolute_url_host(sample: &str) -> bool {
    let Some(after_scheme) = sample
        .strip_prefix("http://")
        .or_else(|| sample.strip_prefix("https://"))
    else {
        return false;
    };
    !after_scheme.contains('/')
}
