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
        if character == '(' {
            if let Some(end) = flag_end(&chars, index) {
                let body: String = chars[index + 2..end].iter().collect();
                apply_verbose(&body, &mut verbose);
                index = end + 1;
                continue;
            }
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

fn flag_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start + 1) != Some(&'?') {
        return None;
    }
    let mut index = start + 2;
    let mut dash = false;
    let mut flag = false;
    while let Some(character) = chars.get(index) {
        match character {
            '-' if !dash => dash = true,
            'i' | 'm' | 's' | 'u' | 'U' | 'x' | 'R' => flag = true,
            ')' if flag => return Some(index),
            _ => return None,
        }
        index += 1;
    }
    None
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
