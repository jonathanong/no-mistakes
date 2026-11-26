use super::control::{
    drop_leading_bangs, drop_leading_word, keyword, unsupported_control, update_double_bracket,
};
use super::{mutations, BranchState, GuardScan};
use crate::codebase::rules::github_actions_download_integrity::words::Word;
use std::ops::Range;

impl GuardScan<'_> {
    pub(super) fn consume_after(
        &mut self,
        idx: usize,
        kw: &str,
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let range = self.slice_range(idx);
        let slice = self.script[range.clone()].trim_start();
        let tail = slice[kw.len()..].trim_start();
        let tail_start = range.end - tail.len();
        let words = drop_leading_word(&self.commands[idx].words, kw);
        self.consume_body(tail_start..range.end, words, record, state)
    }

    pub(super) fn consume_nested(
        &mut self,
        idx: usize,
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let range = self.slice_range(idx);
        let slice = self.script[range.clone()].trim_start();
        let kw = match keyword(slice) {
            "then" => Some("then"),
            "else" => Some("else"),
            "elif" if state.depth > 0 => Some("elif"),
            _ => None,
        };
        let (tail, words) = match kw {
            Some(kw) => (
                slice[kw.len()..].trim_start(),
                drop_leading_word(&self.commands[idx].words, kw),
            ),
            None => (slice, self.commands[idx].words.as_slice()),
        };
        let tail_start = range.end - tail.len();
        self.consume_body(tail_start..range.end, words, record, state)
    }

    pub(super) fn consume_body(
        &mut self,
        range: Range<usize>,
        mut words: &[Word],
        record: bool,
        state: &mut BranchState,
    ) -> bool {
        let was_in_bracket = state.in_double_bracket;
        update_double_bracket(words, &mut state.in_double_bracket);
        let mut slice = &self.script[range.clone()];
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
                        let tail_start = range.end - slice.len();
                        if record && self.has_complex_expansion(tail_start..range.end) {
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
                        let start = range.end - slice.len();
                        mutations::record_mutations(
                            self.script,
                            start..range.end,
                            &self.heredoc_bodies,
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
