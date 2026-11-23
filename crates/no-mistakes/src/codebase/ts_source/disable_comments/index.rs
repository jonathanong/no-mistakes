use super::{
    directives, named_rule, next_line_rule_part, scan_file_directives, DisableDirective,
    LineCommentScanState,
};
use crate::diagnostics::InvocationObserver;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) struct DisableCommentIndex<S> {
    source: S,
    header: OnceLock<HashMap<Box<str>, u32>>,
    lines: [OnceLock<Mutex<LineDirectives>>; 2],
}

impl<S: AsRef<str>> DisableCommentIndex<S> {
    pub(crate) fn new(source: S) -> Self {
        Self {
            source,
            header: OnceLock::new(),
            lines: Default::default(),
        }
    }

    pub(crate) fn matching(
        &self,
        rule: &str,
        line: Option<u32>,
        track_quotes: bool,
        observer: Option<&InvocationObserver>,
    ) -> Option<DisableDirective> {
        let source = self.source.as_ref();
        let header = self.header.get_or_init(|| {
            record(observer, "suppression.header_builds", 1);
            let mut header = HashMap::new();
            scan_file_directives(source, |line, part| {
                let rule = named_rule(part);
                if !rule.is_empty() && !header.contains_key(rule) {
                    header.insert(rule.into(), line);
                }
                false
            });
            header
        });
        if let Some(&line) = header.get(rule) {
            return Some(DisableDirective::File { line });
        }
        if rule.is_empty() || named_rule(rule) != rule {
            return directives::matching_disable_directive_tracking(
                source,
                line,
                rule,
                track_quotes,
            );
        }
        let line = line.filter(|&line| line != 0)?;
        let mut lines = self.lines[usize::from(track_quotes)]
            .get_or_init(|| {
                record(observer, "suppression.line_index_builds", 1);
                Mutex::new(LineDirectives::new(source, track_quotes))
            })
            .lock()
            .expect("disable comment index lock poisoned");
        lines.scan_through(source, line, observer);
        lines.matching(rule, line)
    }
}

#[derive(Default)]
struct LineDirectives {
    next: FxHashMap<u32, Box<str>>,
    inline: FxHashMap<u32, Box<str>>,
    scan_inline: bool,
    scan_next: bool,
    state: LineCommentScanState,
    offset: usize,
    scanned_line: u32,
}

impl LineDirectives {
    fn new(source: &str, track_quotes: bool) -> Self {
        let (scan_inline, scan_next) = line_markers(source);
        Self {
            scan_inline,
            scan_next,
            state: LineCommentScanState {
                track_quotes,
                ..Default::default()
            },
            offset: source.len() - source.trim_start_matches('\u{FEFF}').len(),
            ..Default::default()
        }
    }

    fn scan_through(&mut self, source: &str, line: u32, observer: Option<&InvocationObserver>) {
        let start_line = self.scanned_line;
        let remaining = line.saturating_sub(start_line);
        if remaining == 0 || self.offset == source.len() || !(self.scan_inline || self.scan_next) {
            return;
        }
        for raw in source[self.offset..]
            .split_inclusive('\n')
            .take(remaining as usize)
        {
            self.offset += raw.len();
            self.scanned_line += 1;
            let text = raw
                .strip_suffix('\n')
                .map_or(raw, |text| text.strip_suffix('\r').unwrap_or(text));
            if self.scan_inline {
                if let Some(part) = directives::inline_rule_part(text, &mut self.state) {
                    insert_named_rule(&mut self.inline, self.scanned_line, part);
                }
            }
            if self.scan_next {
                if let Some(part) = next_line_rule_part(text) {
                    insert_named_rule(&mut self.next, self.scanned_line, part);
                }
            }
        }
        let visits = u64::from(self.scanned_line - start_line);
        record(observer, "suppression.line_visits", visits);
        if self.scan_inline {
            record(observer, "suppression.lexer_line_visits", visits);
        }
    }

    fn matching(&self, rule: &str, line: u32) -> Option<DisableDirective> {
        let previous = line - 1;
        if previous != 0 && self.next.get(&previous).is_some_and(|name| &**name == rule) {
            return Some(DisableDirective::NextLine { line: previous });
        }
        self.inline
            .get(&line)
            .is_some_and(|name| &**name == rule)
            .then_some(DisableDirective::Line { line })
    }
}

fn insert_named_rule(rules: &mut FxHashMap<u32, Box<str>>, line: u32, part: &str) {
    let rule = named_rule(part);
    if !rule.is_empty() {
        rules.insert(line, rule.into());
    }
}

fn line_markers(source: &str) -> (bool, bool) {
    const PREFIX: &[u8] = b"no-mistakes-disable-";
    let bytes = source.as_bytes();
    let mut inline = false;
    let mut next = false;
    for start in memchr::memmem::find_iter(bytes, PREFIX) {
        let suffix = &bytes[start + PREFIX.len()..];
        inline |= suffix.starts_with(b"line ");
        next |= suffix.starts_with(b"next-line ");
        if inline && next {
            break;
        }
    }
    (inline, next)
}

fn record(observer: Option<&InvocationObserver>, metric: &'static str, count: u64) {
    if let Some(observer) = observer {
        observer.increment(metric, count);
    }
}
