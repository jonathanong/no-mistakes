use super::flow_scope::inside_flow;
use super::synthetic::is_synthetic;
use super::{finding, CompiledPattern, RuleFinding};
use regex::{Match, Regex};
use std::sync::LazyLock;

/// The text before a pin when the pin is the tail of a `uses:` value: the key
/// (bare, quoted, or escaped), an optional opening quote, then any `owner/` path
/// components, the shape the exact-action-ref pattern reads.
/// `uses` is a key only where a key can start: the line start, a JavaScript
/// `\n`/`\r`/`\t` escape, the opening quote of a string, a flow-mapping `{` or
/// `[`, or a `,` that separates entries inside one (see `inside_flow`), then
/// indentation and an optional `- ` list marker. Prose
/// (`Homebrew uses: homebrew/core/postgresql@18`, `Homebrew, uses: ...`) and a
/// name that only ends in `uses` (`package.uses:`, `steps/uses:`, `$uses:`) are
/// not keys.
static USES_VALUE_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:^|\\[nrt]|["'`{\[]|(?P<comma>,))\s*(?:-\s+)?uses\\?["']?\s*:\s*\\?["']?(?:[\w.-]+/)*$"#)
        .expect("uses value prefix regex")
});

/// How a pin relates to another pin that reads the same text.
#[derive(Clone, Copy, PartialEq)]
enum Reading {
    /// The pattern runs on any line.
    Free,
    /// The pattern needs a line context, so it knows what its text is and owns
    /// the span.
    Owns,
    /// A line-context pin that is the tail of a `uses:` value. It is the formula
    /// reading of what is probably an action ref, so it gives way to a pin that
    /// reads the text as an action ref, and stands when nothing else does.
    Yields,
}

/// A finding and, for a line-scanned pin, the byte span it covers in the file.
pub(super) struct Found {
    finding: RuleFinding,
    span: Option<(usize, usize)>,
    reading: Reading,
}

impl Found {
    pub(super) fn whole(finding: RuleFinding) -> Self {
        Self {
            finding,
            span: None,
            reading: Reading::Free,
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
            let reading = if pattern.line_context.is_none() {
                Reading::Free
            } else if ends_uses_value(&line[..pin.start()]) {
                Reading::Yields
            } else {
                Reading::Owns
            };
            found.push(Found {
                finding: finding(file, index + 1, pattern, pin.as_str()),
                span: Some((offset + pin.start(), offset + pin.end())),
                reading,
            });
        }
        offset += line_with_ending.len();
    }
}

/// A pattern that needs a line context knows what its text is, so its pin owns
/// that span. A context-free pin overlapping it is a second reading of the same
/// text (the action ref `core/postgresql@18` inside the Homebrew formula
/// `homebrew/core/postgresql@18`) and is dropped.
///
/// A line-context pin that is the tail of a `uses:` value owns nothing. It is
/// dropped only when a context-free pin overlaps it, which is the case when the
/// value is an action ref (`uses: Homebrew/actions/setup-homebrew@4`). When no
/// context-free pin reads the text, it is the only finding for it and stays
/// (`uses: Homebrew/core/libc++@18` has a `+`, which an action name cannot
/// contain). Deciding whether `uses` is a key therefore only picks which reason
/// a pin is reported under; it never makes a finding disappear.
pub(super) fn into_findings(found: Vec<Found>) -> Vec<RuleFinding> {
    let spans = |wanted: Reading| -> Vec<(usize, usize)> {
        found
            .iter()
            .filter(|pin| pin.reading == wanted)
            .filter_map(|pin| pin.span)
            .collect()
    };
    let owned = spans(Reading::Owns);
    let free = spans(Reading::Free);
    found
        .into_iter()
        .filter(|pin| match pin.reading {
            Reading::Free => !overlaps_any(pin.span, &owned),
            Reading::Owns => true,
            Reading::Yields => !overlaps_any(pin.span, &free),
        })
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
        let versions = captures.name("versions").unwrap_or(pin);
        if !follows_at(line, pattern, &pin) && !is_synthetic(pin.as_str(), versions.as_str()) {
            pins.push(pin);
        }
    }
    pins
}

/// True when the text before a pin ends in a `uses:` key and the start of its value.
fn ends_uses_value(before: &str) -> bool {
    USES_VALUE_PREFIX.captures(before).is_some_and(|key| {
        key.name("comma")
            .is_none_or(|comma| inside_flow(&before[..comma.start()]))
    })
}

fn follows_at(line: &str, pattern: &CompiledPattern, matched: &Match<'_>) -> bool {
    pattern.reject_preceding_at
        && matched.start() > 0
        && line.as_bytes()[matched.start() - 1] == b'@'
}
