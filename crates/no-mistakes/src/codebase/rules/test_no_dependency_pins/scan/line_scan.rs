use super::synthetic::is_synthetic;
use super::{finding, CompiledPattern, RuleFinding};
use regex::Match;

pub(super) fn scan_lines(
    file: &str,
    content: &str,
    pattern: &CompiledPattern,
    findings: &mut Vec<RuleFinding>,
) {
    for (index, line_with_ending) in content.split_inclusive('\n').enumerate() {
        let line = line_with_ending.trim_end_matches(['\r', '\n']);
        let pins = if pattern.builtin {
            builtin_pins(line, pattern)
        } else {
            custom_pins(line, pattern)
        };
        for pin in pins {
            findings.push(finding(file, index + 1, pattern, pin.as_str()));
        }
    }
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
