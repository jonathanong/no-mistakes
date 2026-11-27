use super::env_read;
use crate::codebase::rules::github_actions_download_integrity::words::{parse, Command};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

mod branch;
mod consume;
mod control;
mod eval;
mod mutations;
mod scan;

use control::{keyword, unsupported_control, update_double_bracket};

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
    if parsed.unterminated_quote || parsed.unterminated_heredoc || parsed.unterminated_substitution
    {
        return Vec::new();
    }
    let positions: Vec<_> = script.char_indices().map(|(index, _)| index).collect();
    let to_byte = |pos: usize| positions.get(pos).copied().unwrap_or(script.len());
    let heredocs: Vec<_> = parsed
        .heredoc_ranges
        .iter()
        .map(|range| to_byte(range.start)..to_byte(range.end))
        .collect();
    let heredoc_bodies: Vec<_> = parsed
        .heredoc_bodies
        .iter()
        .map(|body| scan::HeredocSpan {
            range: to_byte(body.range.start)..to_byte(body.range.end),
            quoted: body.quoted,
        })
        .collect();
    let mut scan = GuardScan {
        script,
        commands: &parsed.commands,
        positions: &positions,
        heredocs,
        heredoc_bodies,
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

struct GuardScan<'a> {
    script: &'a str,
    commands: &'a [Command],
    positions: &'a [usize],
    heredocs: Vec<Range<usize>>,
    heredoc_bodies: Vec<scan::HeredocSpan>,
    env: &'a BTreeMap<String, String>,
    mutated_vars: BTreeSet<String>,
    mutated_all: bool,
    excluded: Vec<Range<usize>>,
}

struct BranchState {
    depth: usize,
    in_double_bracket: bool,
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
                self.script,
                self.slice_range(idx),
                &self.heredoc_bodies,
                &self.commands[idx].words,
                &mut self.mutated_vars,
                &mut self.mutated_all,
            );
            idx += 1;
        }
    }

    fn slice_range(&self, idx: usize) -> Range<usize> {
        let start = self.positions[self.commands[idx].anchor];
        let end = self
            .commands
            .get(idx + 1)
            .map_or(self.script.len(), |next| self.positions[next.anchor]);
        start..end
    }

    fn slice(&self, idx: usize) -> &'a str {
        self.script[self.slice_range(idx)].trim_start()
    }

    fn has_arithmetic_at(&self, idx: usize) -> bool {
        scan::has_arithmetic_in_range(self.script, self.slice_range(idx), &self.heredocs)
    }

    fn has_complex_expansion(&self, range: Range<usize>) -> bool {
        scan::has_complex_expansion_in_range(self.script, range, &self.heredoc_bodies)
    }
}
