use super::DisableDirective;
use super::{comment_body, line_comment_start, LineCommentScanState};

/// A supported directive and its single canonical rule identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedDisableDirective {
    pub rule: String,
    pub directive: DisableDirective,
}

/// Uses the filtering lexer and grammar, so strings and block comments cannot
/// create audit-only directives. Markdown keeps the existing prose quote mode.
#[doc(hidden)]
pub fn disable_directives(source: &str, track_quotes: bool) -> Vec<NamedDisableDirective> {
    if !source.contains("no-mistakes-disable-") {
        return Vec::new();
    }
    let mut state = LineCommentScanState {
        track_quotes,
        ..Default::default()
    };
    let mut found = Vec::new();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let Some((start, kind)) = line_comment_start(line, &mut state) else {
            continue;
        };
        let body = comment_body(line, start, kind);
        let line_number = (index + 1) as u32;
        let Some((suffix, rest)) = body
            .strip_prefix("no-mistakes-disable-")
            .and_then(|rest| rest.split_once(' '))
        else {
            continue;
        };
        let rule = rest
            .trim()
            .split(|ch: char| ch == ':' || ch.is_whitespace())
            .next()
            .unwrap_or("");
        if rule.is_empty() {
            continue;
        }
        let directive = match suffix {
            "line" => DisableDirective::Line { line: line_number },
            "next-line"
                if line[..start].trim().is_empty()
                    && super::leading_comment_text(line.trim()).is_some() =>
            {
                DisableDirective::NextLine { line: line_number }
            }
            "file"
                if super::disable_file_directive_line_at(source, rule, Some(line_number))
                    .is_some() =>
            {
                DisableDirective::File { line: line_number }
            }
            _ => continue,
        };
        found.push(NamedDisableDirective {
            rule: rule.to_owned(),
            directive,
        });
    }
    found
}
