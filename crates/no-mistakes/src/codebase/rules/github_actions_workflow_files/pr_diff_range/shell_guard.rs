use super::env_read;
use crate::codebase::rules::github_actions_download_integrity::words::{parse, Command, Word};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

mod branch;
mod control;
mod eval;
mod mutations;
mod scan;

use control::{drop_leading_bangs, drop_leading_word, keyword, unsupported_control, update_double_bracket};

pub(super) fn excluded_on_merge_group(
    script: &str,
    env: &BTreeMap<String, String>,
) -> Vec<Range<usize>> {
    if !script
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ';' | '|' | '&' | '(' | ')'))
        .any(|word| word == "if")
    {
        return Vec::new();
    }
    let parsed = parse(script);
    if parsed.unterminated_quote || parsed.unterminated_heredoc || parsed.unterminated_substitution {
        return Vec::new();
    }
    let positions: Vec<_> = script.char_indices().map(|(index, _)| index).collect();
    let heredocs: Vec<_> = parsed
        .heredoc_ranges
        .iter()
        .map(|range| {
            let start = positions.get(range.start).copied().unwrap_or(script.len());
            let end = positions.get(range.end).copied().unwrap_or(script.len());
            start..end
        })
        .collect();
    let mut scan = GuardScan {
        script,
        commands: &parsed.commands,
        positions: &positions,
        heredocs,
        env,
        mutated_vars: BTreeSet::new(),
        mutated_all: false,
        excluded: Vec::new(),
    };
    scan.run();
    scan.excluded
}

pub(super) fn references_outside(script: &str, name: &str, excluded: &[Range<usize>]) -> bool {
    let mut cursor = 0;
    for span in excluded {
        if span.start > cursor && env_read::references(&script[cursor..span.start], name) {
            return true;
        }
        cursor = cursor.max(span.end);
    }
    env_read::references(&script[cursor..], name)
}

pub(super) struct GuardScan<'a> {
    pub(super) script: &'a str,
    pub(super) commands: &'a [Command],
    pub(super) positions: &'a [usize],
    pub(super) heredocs: Vec<Range<usize>>,
    pub(super) env: &'a BTreeMap<String, String>,
    pub(super) mutated_vars: BTreeSet<String>,
    pub(super) mutated_all: bool,
    pub(super) excluded: Vec<Range<usize>>,
}

pub(super) struct BranchState {
    pub(super) depth: usize,
    pub(super) in_double_bracket: bool,
}

impl<'a> GuardScan<'a> {
    fn run(&mut self) {
        if self.commands.first().is_some_and(|first| {
            scan::has_arithmetic_in_range(
                self.script,
                0..self.positions[first.anchor],
                &self.heredocs,
            )
        }) {
            return;
        }
        let mut in_double_bracket = false;
        let mut idx = 0;
        while idx < self.commands.len() {
            let slice = self.slice(idx);
            if keyword(slice) == "if" {
                let Some(next) = self.scan_if(idx) else {
                    break;
                };
                idx = next;
                continue;
            }
            update_double_bracket(&self.commands[idx].words, &mut in_double_bracket);
            if unsupported_control(slice, in_double_bracket) || self.has_arithmetic_at(idx) {
                break;
            }
            mutations::record_mutations(
                slice,
                &self.commands[idx].words,
                &mut self.mutated_vars,
                &mut self.mutated_all,
            );
            idx += 1;
        }
    }

    pub(super) fn slice_range(&self, idx: usize) -> Range<usize> {
        let start = self.positions[self.commands[idx].anchor];
        let end = self
            .commands
            .get(idx + 1)
            .map_or(self.script.len(), |next| self.positions[next.anchor]);
        start..end
    }

    pub(super) fn slice(&self, idx: usize) -> &'a str {
        self.script[self.slice_range(idx)].trim_start()
    }

    pub(super) fn has_arithmetic_at(&self, idx: usize) -> bool {
        scan::has_arithmetic_in_range(self.script, self.slice_range(idx), &self.heredocs)
    }

    pub(super) fn consume_nested(
        &mut self,
        slice: &str,
        words: &[Word],
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let (slice, words) = match keyword(slice) {
            "then" => (slice[4..].trim_start(), drop_leading_word(words, "then")),
            "else" => (slice[4..].trim_start(), drop_leading_word(words, "else")),
            "elif" if state.depth > 0 => (slice[4..].trim_start(), drop_leading_word(words, "elif")),
            _ => (slice, words),
        };
        self.consume_body(slice, words, record, state)
    }

    pub(super) fn consume_body(
        &mut self,
        mut slice: &str,
        mut words: &[Word],
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let was_in_bracket = state.in_double_bracket;
        update_double_bracket(words, &mut state.in_double_bracket);
        loop {
            let trimmed = slice
                .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '|' | '&' | '!'));
            match keyword(trimmed) {
                "if" => {
                    state.depth += 1;
                    slice = trimmed[2..].trim_start();
                    words = drop_leading_word(drop_leading_bangs(words), "if");
                }
                "fi" if state.depth > 0 => {
                    state.depth -= 1;
                    slice = trimmed[2..].trim_start();
                    words = drop_leading_word(drop_leading_bangs(words), "fi");
                    if words.is_empty() {
                        if record && scan::has_complex_expansion(slice) {
                            self.mutated_all = true;
                        }
                        return true;
                    }
                }
                _ => {
                    if unsupported_control(trimmed, was_in_bracket) {
                        return false;
                    }
                    if record {
                        mutations::record_mutations(
                            slice,
                            words,
                            &mut self.mutated_vars,
                            &mut self.mutated_all,
                        );
                    }
                    return true;
                }
            }
        }
    }
}
