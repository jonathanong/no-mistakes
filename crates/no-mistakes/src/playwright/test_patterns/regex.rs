use oxc_ast::ast::RegExpFlags;
use regex::{Regex, RegexBuilder};
use std::iter::Peekable;
use std::str::Chars;

pub(super) fn compile(pattern: &str, flags: RegExpFlags) -> Option<Regex> {
    if flags.contains(RegExpFlags::V) {
        return None;
    }
    let ascii_case = flags.contains(RegExpFlags::I) && !flags.contains(RegExpFlags::U);
    let mut normalized = normalize(pattern, flags, ascii_case)?;
    // Playwright resets lastIndex before each file, including sticky regexes.
    if flags.contains(RegExpFlags::Y) {
        normalized = format!(r"\A(?:{normalized})");
    }
    RegexBuilder::new(&normalized)
        .case_insensitive(flags.contains(RegExpFlags::I) && !ascii_case)
        .multi_line(flags.contains(RegExpFlags::M))
        .dot_matches_new_line(flags.contains(RegExpFlags::S))
        .build()
        .ok()
}

fn normalize(pattern: &str, flags: RegExpFlags, ascii_case: bool) -> Option<String> {
    let unicode = flags.contains(RegExpFlags::U);
    let mut normalized = String::new();
    let mut depth = 0usize;
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => append_escape(&mut chars, &mut normalized)?,
            '[' => {
                let class = character_class(&mut chars, unicode)?;
                append_class(&class, ascii_case, &mut normalized);
            }
            '.' => normalized.push_str(wildcard(chars.peek().copied(), depth, flags)?),
            '$' => {
                require_end_anchor(chars.peek().is_some(), flags)?;
                normalized.push_str(r"\z");
            }
            '^' if flags.contains(RegExpFlags::M) => return None,
            '(' => {
                depth += 1;
                normalized.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                normalized.push(ch);
            }
            _ => append_literal(ch, unicode, ascii_case, &mut normalized)?,
        }
    }
    Some(normalized)
}

fn append_escape(chars: &mut Peekable<Chars<'_>>, normalized: &mut String) -> Option<()> {
    let escaped = chars.next()?;
    // These escapes have the same literal meaning in both engines.
    // Character classes, identity escapes and Unicode escapes do not.
    if !"nrtf/\\.[]{}()*+?|^$-".contains(escaped) {
        return None;
    }
    normalized.push('\\');
    normalized.push(escaped);
    Some(())
}

fn character_class(chars: &mut Peekable<Chars<'_>>, unicode: bool) -> Option<String> {
    if !unicode && chars.peek() == Some(&'^') {
        return None;
    }
    let mut class = String::from("[");
    loop {
        let next = chars.next()?;
        if next == '[' || next == '\\' || (!unicode && !next.is_ascii()) {
            return None;
        }
        class.push(next);
        if next == ']' {
            break;
        }
    }
    if class.contains("&&") || class.contains("~~") || class.contains("--") {
        return None;
    }
    Some(class)
}

fn append_class(class: &str, ascii_case: bool, normalized: &mut String) {
    if ascii_case {
        normalized.push_str(&format!("(?i-u:{class})"));
    } else {
        normalized.push_str(class);
    }
}

fn wildcard(next: Option<char>, depth: usize, flags: RegExpFlags) -> Option<&'static str> {
    // Non-u JS regexes count UTF-16 code units. Only ungrouped
    // wildcard runs are compatible with scalar matching on paths.
    if !flags.contains(RegExpFlags::U) && (depth != 0 || !matches!(next, Some('*' | '+'))) {
        return None;
    }
    if flags.contains(RegExpFlags::S) {
        return Some(".");
    }
    // ECMAScript's dot excludes all four line terminators.
    Some(r"[^\n\r\u{2028}\u{2029}]")
}

fn require_end_anchor(has_next: bool, flags: RegExpFlags) -> Option<()> {
    // Without m, JS requires the actual end of the input. Mid-pattern
    // and multiline anchors stay outside the compatible subset.
    if has_next || flags.contains(RegExpFlags::M) {
        return None;
    }
    Some(())
}

fn append_literal(
    ch: char,
    unicode: bool,
    ascii_case: bool,
    normalized: &mut String,
) -> Option<()> {
    if !unicode && !ch.is_ascii() {
        return None;
    }
    if ascii_case && ch.is_ascii_alphabetic() {
        normalized.push('[');
        normalized.push(ch.to_ascii_lowercase());
        normalized.push(ch.to_ascii_uppercase());
        normalized.push(']');
    } else {
        normalized.push(ch);
    }
    Some(())
}
