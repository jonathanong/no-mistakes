use super::synthetic::is_synthetic;
use super::{finding, CompiledPattern, RuleFinding};
use regex::Match;

/// A finding and, for a line-scanned pin, the byte span it covers in the file.
pub(super) struct Found {
    finding: RuleFinding,
    span: Option<(usize, usize)>,
    /// The pattern only runs on lines with a `line_context`.
    scoped: bool,
}

impl Found {
    pub(super) fn whole(finding: RuleFinding) -> Self {
        Self {
            finding,
            span: None,
            scoped: false,
        }
    }
}

pub(super) fn scan_lines(
    file: &str,
    content: &str,
    pattern: &CompiledPattern,
    found: &mut Vec<Found>,
) {
    let mut offset = 0;
    for (index, line_with_ending) in content.split_inclusive('\n').enumerate() {
        let line = line_with_ending.trim_end_matches(['\r', '\n']);
        let pins = if pattern.builtin {
            builtin_pins(line, pattern)
        } else {
            custom_pins(line, pattern)
        };
        for pin in pins {
            found.push(Found {
                finding: finding(file, index + 1, pattern, pin.as_str()),
                span: Some((offset + pin.start(), offset + pin.end())),
                scoped: pattern.line_context.is_some(),
            });
        }
        offset += line_with_ending.len();
    }
}

/// A pattern that needs a line context knows what its text is, so its pin owns
/// that span. A context-free pin overlapping it is a second reading of the same
/// text (the action ref `core/postgresql@18` inside the Homebrew formula
/// `homebrew/core/postgresql@18`) and is dropped.
pub(super) fn into_findings(found: Vec<Found>) -> Vec<RuleFinding> {
    let owned: Vec<(usize, usize)> = found
        .iter()
        .filter(|pin| pin.scoped)
        .filter_map(|pin| pin.span)
        .collect();
    found
        .into_iter()
        .filter(|pin| pin.scoped || !overlaps_any(pin.span, &owned))
        .map(|pin| pin.finding)
        .collect()
}

fn overlaps_any(span: Option<(usize, usize)>, owned: &[(usize, usize)]) -> bool {
    span.is_some_and(|(start, end)| {
        owned
            .iter()
            .any(|&(owned_start, owned_end)| start < owned_end && owned_start < end)
    })
}

/// Custom patterns are literal: every regex match is a pin.
fn custom_pins<'l>(line: &'l str, pattern: &CompiledPattern) -> Vec<Match<'l>> {
    pattern
        .regex
        .find_iter(line)
        .filter(|matched| !follows_at(line, pattern, matched))
        .collect()
}

/// Default patterns report the `pin` capture (or the whole match), skip
/// placeholder values, and only run on lines matching the pattern's context.
///
/// The search resumes at the end of the pin rather than the end of the match,
/// so the separator that closed one pin can open the next.
fn builtin_pins<'l>(line: &'l str, pattern: &CompiledPattern) -> Vec<Match<'l>> {
    if pattern
        .line_context
        .as_ref()
        .is_some_and(|context| !context.is_match(line))
    {
        return Vec::new();
    }
    let mut pins = Vec::new();
    let mut at = 0;
    while let Some(captures) = pattern.regex.captures_at(line, at) {
        let pin = captures
            .name("pin")
            .or_else(|| captures.get(0))
            .expect("a regex match has a full match");
        at = pin.end();
        if !follows_at(line, pattern, &pin) && !is_synthetic(pin.as_str()) {
            pins.push(pin);
        }
    }
    pins
}

fn follows_at(line: &str, pattern: &CompiledPattern, matched: &Match<'_>) -> bool {
    pattern.reject_preceding_at
        && matched.start() > 0
        && line.as_bytes()[matched.start() - 1] == b'@'
}
