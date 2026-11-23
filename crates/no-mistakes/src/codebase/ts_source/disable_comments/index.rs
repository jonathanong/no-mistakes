use super::{directives, named_rule, next_line_rule_part, scan_file_directives, DisableDirective};
use crate::diagnostics::InvocationObserver;
use rustc_hash::FxHashMap;
use std::sync::OnceLock;

pub(crate) struct DisableCommentIndex<S> {
    source: S,
    header: OnceLock<FxHashMap<Box<str>, u32>>,
    lines: [OnceLock<LineDirectives>; 2],
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
            let mut header = FxHashMap::default();
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
        self.lines[usize::from(track_quotes)]
            .get_or_init(|| LineDirectives::scan(source, track_quotes, observer))
            .matching(rule, line)
    }
}

#[derive(Default)]
struct LineDirectives {
    next: FxHashMap<u32, Box<str>>,
    inline: FxHashMap<u32, Box<str>>,
}

impl LineDirectives {
    fn scan(source: &str, track_quotes: bool, observer: Option<&InvocationObserver>) -> Self {
        record(observer, "suppression.line_index_builds", 1);
        let (scan_inline, scan_next) = line_markers(source);
        let mut found = Self::default();
        if !scan_inline && !scan_next {
            return found;
        }

        let mut state = super::LineCommentScanState {
            track_quotes,
            ..Default::default()
        };
        let mut visits = 0;
        for (index, text) in source
            .trim_start_matches('\u{FEFF}')
            .lines()
            .take(u32::MAX as usize)
            .enumerate() {
            let line = (index + 1) as u32;
            if scan_inline {
                if let Some(part) = directives::inline_rule_part(text, &mut state) {
                    insert_named_rule(&mut found.inline, line, part);
                }
            }
            if scan_next {
                if let Some(part) = next_line_rule_part(text) {
                    insert_named_rule(&mut found.next, line, part);
                }
            }
            visits = index + 1;
        }
        record(observer, "suppression.line_visits", visits as u64);
        if scan_inline {
            record(observer, "suppression.lexer_line_visits", visits as u64);
        }
        found
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
