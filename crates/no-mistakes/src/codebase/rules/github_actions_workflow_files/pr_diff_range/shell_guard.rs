use super::env_read;
use crate::codebase::rules::github_actions_download_integrity::words::{parse, Command, Word};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

mod branch;
mod eval;

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
    let mut scan = GuardScan {
        script,
        commands: &parsed.commands,
        positions: &positions,
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
            if unsupported_control(slice, in_double_bracket) {
                break;
            }
            eval::record_mutations(
                &self.commands[idx].words,
                &mut self.mutated_vars,
                &mut self.mutated_all,
            );
            idx += 1;
        }
    }

    pub(super) fn slice(&self, idx: usize) -> &'a str {
        let start = self.positions[self.commands[idx].anchor];
        let end = self
            .commands
            .get(idx + 1)
            .map_or(self.script.len(), |next| self.positions[next.anchor]);
        self.script[start..end].trim_start()
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
            _ => (slice, words),
        };
        self.consume_body(slice, words, record, state)
    }

    pub(super) fn consume_body(
        &mut self,
        slice: &str,
        words: &[Word],
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let was_in_bracket = state.in_double_bracket;
        update_double_bracket(words, &mut state.in_double_bracket);
        let trimmed = slice
            .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '|' | '&' | '!'));
        let ok = match keyword(trimmed) {
            "if" => {
                state.depth += 1;
                true
            }
            "fi" if state.depth > 0 => {
                state.depth -= 1;
                true
            }
            "elif" if state.depth > 0 => true,
            _ => !unsupported_control(trimmed, was_in_bracket),
        };
        if ok && record {
            eval::record_mutations(words, &mut self.mutated_vars, &mut self.mutated_all);
        }
        ok
    }
}

pub(super) fn drop_leading_word<'a>(words: &'a [Word], kw: &str) -> &'a [Word] {
    &words[usize::from(words.first().is_some_and(|word| word.text == kw))..]
}

pub(super) fn keyword(source: &str) -> &str {
    source
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ';' | '|' | '&'))
        .next()
        .unwrap_or_default()
}

fn update_double_bracket(words: &[Word], in_double_bracket: &mut bool) {
    for word in words {
        if word.text == "[[" {
            *in_double_bracket = true;
        } else if word.text == "]]" {
            *in_double_bracket = false;
        }
    }
}

fn unsupported_control(source: &str, in_double_bracket: bool) -> bool {
    let source = source
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '|' | '&' | '!'));
    let word = keyword(source);
    (!in_double_bracket && source.starts_with(['(', '{']))
        || matches!(
            word,
            "if" | "elif"
                | "else"
                | "fi"
                | "case"
                | "while"
                | "until"
                | "for"
                | "select"
                | "function"
        )
        || word.ends_with("()")
}
