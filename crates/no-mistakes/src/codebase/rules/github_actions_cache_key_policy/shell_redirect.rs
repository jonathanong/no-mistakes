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
    let Some((start, _)) = chars.get(at).copied() else {
        *index = at;
        return None;
    };
    let (target, next_at) = redirect_word(line, chars, at);
    *index = char_index(chars, next_at);
    (!target.is_empty()).then_some(Hit {
        target,
        start,
        end: next_at,
    })
}

fn char_index(chars: &[(usize, char)], byte: usize) -> usize {
    chars
        .iter()
        .position(|(at, _)| *at >= byte)
        .unwrap_or(chars.len())
}

fn redirect_word(line: &str, chars: &[(usize, char)], at: usize) -> (String, usize) {
    let Some((start, ch)) = chars.get(at).copied() else {
        return (String::new(), line.len());
    };
    let (mut word, mut next_at, quoted) = fragment(line, start, ch);
    let mut literal = quoted && !word.is_empty();
    while let Some((byte, next)) = chars.get(char_index(chars, next_at)).copied() {
        if next.is_whitespace() || is_meta(next) {
            break;
        }
        let (piece, further, piece_quoted) = fragment(line, byte, next);
        if further == next_at {
            break;
        }
        if extends_name(&word, &piece) {
            word.push('.');
        }
        literal |= piece_quoted && !piece.is_empty();
        word.push_str(&piece);
        next_at = further;
    }
    if literal && super::super::output_syntax::var_ref(&word).is_some() {
        word.clear();
    }
    (word, next_at)
}

fn extends_name(word: &str, piece: &str) -> bool {
    super::super::output_syntax::var_ref(word).is_some()
        && piece.starts_with(|ch: char| ch == '_' || ch.is_ascii_alphanumeric())
}

fn fragment(line: &str, start: usize, ch: char) -> (String, usize, bool) {
    if ch == '"' || ch == '\'' {
        let body_at = start + ch.len_utf8();
        let body = &line[body_at..];
        let Some(end) = body.find(ch) else {
            return (String::new(), line.len(), false);
        };
        let body = &body[..end];
        return (body.to_string(), body_at + end + ch.len_utf8(), ch == '\'');
    }
    let (text, next_at) = bare_word(line, start);
    (text, next_at, false)
}

fn bare_fd(line: &str, chars: &[(usize, char)], at: usize) -> bool {
    let Some((start, ch)) = chars.get(at).copied() else {
        return false;
    };
    if ch == '"' || ch == '\'' {
        return false;
    }
    let (token, _) = bare_word(line, start);
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
}

fn bare_word(line: &str, start: usize) -> (String, usize) {
    let rest = &line[start..];
    let mut end = 0;
    let mut token = String::new();
    while end < rest.len() {
        let ch = rest[end..].chars().next().unwrap();
        if ch == '\\' {
            let after = end + ch.len_utf8();
            let Some(escaped) = rest[after..].chars().next() else {
                end = rest.len();
                break;
            };
            if escaped == '$' {
                token.push('\\');
            }
            token.push(escaped);
            end = after + escaped.len_utf8();
            continue;
        }
        if ch.is_whitespace() || is_meta(ch) {
            break;
        }
        token.push(ch);
        end += ch.len_utf8();
    }
    (token, start + end)
}

fn is_meta(ch: char) -> bool {
    matches!(ch, '>' | '<' | '|' | '&' | ';' | '(' | ')')
}
