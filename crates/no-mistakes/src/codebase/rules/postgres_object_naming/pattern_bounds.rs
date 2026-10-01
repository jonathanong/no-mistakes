pub(super) struct Bounds {
    pub anchors: bool,
    pub assertion: bool,
}

pub(super) fn bounds(raw: &str) -> Bounds {
    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0;
    let mut verbose = false;
    let mut comment = false;
    let mut in_class = false;
    let mut started = false;
    let mut caret = false;
    let mut ended = false;
    let mut recent = String::new();
    let mut assertion = false;
    let mut after_table = false;
    let mut scopes: Vec<bool> = Vec::new();
    while index < chars.len() {
        let character = chars[index];
        if comment {
            if character == '\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        if character == '\\' {
            let next = chars.get(index + 1).copied();
            if after_table && matches!(next, Some('b' | 'B' | 'A' | 'G')) {
                assertion = true;
            }
            push_recent(&mut recent, '\\');
            if let Some(next) = next {
                push_recent(&mut recent, next);
            }
            note(&mut started, &mut caret, &mut ended, '\\', false);
            if let Some(next) = next {
                note(&mut started, &mut caret, &mut ended, next, true);
            }
            after_table = false;
            index += 2;
            continue;
        }
        if in_class {
            if character == ']' {
                in_class = false;
            }
            note(&mut started, &mut caret, &mut ended, character, false);
            after_table = false;
            index += 1;
            continue;
        }
        if character == '[' {
            index += 1;
            if chars.get(index) == Some(&'^') {
                index += 1;
            }
            if chars.get(index) == Some(&']') {
                index += 1;
            }
            in_class = true;
            after_table = false;
            continue;
        }
        if character == '#' && verbose {
            comment = true;
            index += 1;
            continue;
        }
        if character == '(' || character == ')' {
            if character == '(' {
                let saved = verbose;
                if open_group(&chars, &mut index, &mut verbose) {
                    scopes.push(saved);
                }
            } else {
                if let Some(previous) = scopes.pop() {
                    verbose = previous;
                }
                index += 1;
            }
            continue;
        }
        if verbose && character.is_whitespace() {
            index += 1;
            continue;
        }
        if raw[byte_at(&chars, index)..].starts_with("{table}") {
            if bad_before(&recent) {
                assertion = true;
            }
            after_table = true;
            recent.clear();
            index += "{table}".chars().count();
            continue;
        }
        if after_table && character == '^' {
            assertion = true;
        }
        push_recent(&mut recent, character);
        note(&mut started, &mut caret, &mut ended, character, false);
        after_table = false;
        index += 1;
    }
    Bounds {
        anchors: caret && ended,
        assertion,
    }
}

fn note(started: &mut bool, caret: &mut bool, ended: &mut bool, character: char, escaped: bool) {
    if !*started {
        *caret = character == '^' && !escaped;
        *started = true;
    }
    *ended = character == '$' && !escaped;
}

fn push_recent(recent: &mut String, character: char) {
    recent.push(character);
    if recent.chars().count() > 2 {
        recent.remove(0);
    }
}

fn bad_before(recent: &str) -> bool {
    ["\\b", "\\B", "\\z", "\\Z", "\\G"]
        .iter()
        .any(|token| recent.ends_with(token))
        || recent.ends_with('$') && !recent.ends_with("\\$")
}

fn byte_at(chars: &[char], index: usize) -> usize {
    chars
        .iter()
        .take(index)
        .map(|character| character.len_utf8())
        .sum()
}

fn open_group(chars: &[char], index: &mut usize, verbose: &mut bool) -> bool {
    let start = *index;
    let mut cursor = start + 1;
    if chars.get(cursor) != Some(&'?') {
        *index = cursor;
        return true;
    }
    let body_at = cursor + 1;
    cursor = body_at;
    let mut dash = false;
    let mut flag = false;
    while let Some(character) = chars.get(cursor).copied() {
        match character {
            ':' | '=' | '!' if !flag && cursor == body_at => {
                *index = cursor + 1;
                return true;
            }
            '-' if !dash => dash = true,
            'i' | 'm' | 's' | 'u' | 'U' | 'x' | 'R' => flag = true,
            ')' if flag => {
                apply_verbose(&chars[body_at..cursor].iter().collect::<String>(), verbose);
                *index = cursor + 1;
                return false;
            }
            ':' if flag => {
                apply_verbose(&chars[body_at..cursor].iter().collect::<String>(), verbose);
                *index = cursor + 1;
                return true;
            }
            _ => break,
        }
        cursor += 1;
    }
    *index = start + 1;
    true
}

fn apply_verbose(body: &str, verbose: &mut bool) {
    let (on, off) = body.split_once('-').unwrap_or((body, ""));
    if on.contains('x') {
        *verbose = true;
    }
    if off.contains('x') {
        *verbose = false;
    }
}
