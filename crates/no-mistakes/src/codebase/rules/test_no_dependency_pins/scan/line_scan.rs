use super::synthetic::is_synthetic;
use super::{finding, CompiledPattern, RuleFinding};
use regex::{Match, Regex};
use std::sync::LazyLock;

/// The text before a pin when the pin ends a `uses:` value: the key (bare,
/// quoted, or escaped), an optional opening quote, then one or more `owner/`
/// path components, the shape the exact-action-ref pattern reads. A slashless
/// value (`uses: postgresql@18`) is not an action ref, so it stays a formula.
/// `uses` is a key only where a key can start: the line start, a JavaScript
/// `\n`/`\r`/`\t` escape, the opening quote of a string, or a flow-mapping `{`
/// or `[` (a `,` starts a key only after one, so `{ name: x, uses: ... }` is a
/// key but `Homebrew, uses: ...` is not), then indentation and an optional `- `
/// list marker. Prose (`Homebrew uses: homebrew/core/postgresql@18`) and a name
/// that only ends in `uses` (`package.uses:`, `steps/uses:`, `$uses:`) are not
/// keys.
static USES_VALUE_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:^|\\[nrt]|["'`]|[{\[](?:[^}\],]*,)*)\s*(?:-\s+)?uses\\?["']?\s*:\s*\\?["']?(?:[\w.-]+/)+$"#)
        .expect("uses value prefix regex")
});

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
/// `homebrew/core/postgresql@18`) and is dropped. A pin that is not claimed in
/// the first place (see `builtin_pins`) leaves the span to the context-free
/// pin.
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
///
/// A pattern with a line context never claims the tail of a `uses:` value: that
/// text is an action ref, and `uses: Homebrew/actions/setup-homebrew@4` is not
/// the formula `setup-homebrew@4` even though the line mentions Homebrew. A
/// tap-qualified formula (`brew install homebrew/core/postgresql@18`) has no
/// `uses:` key in front of it and is still claimed.
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
        if !follows_at(line, pattern, &pin)
            && !is_synthetic(pin.as_str())
            && !ends_uses_value(line, pattern, &pin)
        {
            pins.push(pin);
        }
    }
    pins
}

/// True for a pin of a line-context pattern that ends a `uses:` value.
fn ends_uses_value(line: &str, pattern: &CompiledPattern, pin: &Match<'_>) -> bool {
    pattern.line_context.is_some() && USES_VALUE_PREFIX.is_match(&line[..pin.start()])
}

fn follows_at(line: &str, pattern: &CompiledPattern, matched: &Match<'_>) -> bool {
    pattern.reject_preceding_at
        && matched.start() > 0
        && line.as_bytes()[matched.start() - 1] == b'@'
}
