use oxc_ast::ast::RegExpFlags;
use regex::{Regex, RegexBuilder};

pub(super) fn compile(pattern: &str, flags: RegExpFlags) -> Option<Regex> {
    if flags.contains(RegExpFlags::V) {
        return None;
    }
    let ascii_case = flags.contains(RegExpFlags::I) && !flags.contains(RegExpFlags::U);
    let mut normalized = String::new();
    let unicode = flags.contains(RegExpFlags::U);
    let mut depth = 0usize;
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let escaped = chars.next()?;
                // These escapes have the same literal meaning in both engines.
                // Character classes, identity escapes and Unicode escapes do not.
                if !"nrtf/\\.[]{}()*+?|^$-".contains(escaped) {
                    return None;
                }
                normalized.push('\\');
                normalized.push(escaped);
            }
            '[' => {
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
                if ascii_case {
                    normalized.push_str(&format!("(?i-u:{class})"));
                } else {
                    normalized.push_str(&class);
                }
            }
            '.' => {
                // Non-u JS regexes count UTF-16 code units. Only ungrouped
                // wildcard runs are compatible with scalar matching on paths.
                if !unicode && (depth != 0 || !matches!(chars.peek(), Some('*' | '+'))) {
                    return None;
                }
                if flags.contains(RegExpFlags::S) {
                    normalized.push('.');
                    continue;
                }
                // ECMAScript's dot excludes all four line terminators.
                normalized.push_str(r"[^\n\r\u{2028}\u{2029}]");
            }
            '$' => {
                // Without m, JS requires the actual end of the input. Mid-pattern
                // and multiline anchors stay outside the compatible subset.
                if chars.peek().is_some() || flags.contains(RegExpFlags::M) {
                    return None;
                }
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
            non_ascii if !unicode && !non_ascii.is_ascii() => return None,
            letter if ascii_case && letter.is_ascii_alphabetic() => {
                normalized.push('[');
                normalized.push(letter.to_ascii_lowercase());
                normalized.push(letter.to_ascii_uppercase());
                normalized.push(']');
            }
            _ => normalized.push(ch),
        }
    }
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
