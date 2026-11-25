use super::eval::{eval_condition, OnMergeGroup};
use super::scan::{self, keyword};
use super::{drop_leading_word, BranchState, GuardScan};

enum Phase {
    InCondition {
        cond_start: usize,
        cond_cmd_idx: usize,
        prev_already_taken: bool,
    },
    InBranch {
        body_start: usize,
        excluded: bool,
    },
}

impl GuardScan<'_> {
    pub(super) fn scan_if(&mut self, start_idx: usize) -> Option<usize> {
        let mut phase = Phase::InCondition {
            cond_start: self.positions[self.commands[start_idx].anchor],
            cond_cmd_idx: start_idx,
            prev_already_taken: false,
        };
        let mut already_taken = false;
        let mut saw_else = false;
        let mut state = BranchState {
            depth: 0,
            in_double_bracket: false,
        };
        let start_slice = self.slice(start_idx);
        let start_words = drop_leading_word(&self.commands[start_idx].words, "if");
        if !self.consume_body(start_slice[2..].trim_start(), start_words, true, &mut state) {
            return None;
        }
        let mut pending = Vec::new();
        let mut idx = start_idx + 1;
        while idx < self.commands.len() {
            let cmd = &self.commands[idx];
            let cmd_start = self.positions[cmd.anchor];
            let slice = self.slice(idx);
            match phase {
                Phase::InCondition {
                    cond_start,
                    cond_cmd_idx,
                    prev_already_taken,
                } => {
                    if state.depth == 0 && keyword(slice) == "then" {
                        let excluded = self.judge_condition(
                            cond_start,
                            cond_cmd_idx,
                            idx,
                            cmd_start,
                            prev_already_taken,
                            &mut already_taken,
                        );
                        let body_start = if prev_already_taken {
                            cond_start
                        } else {
                            cmd_start
                        };
                        let words = drop_leading_word(&cmd.words, "then");
                        if !self.consume_body(slice[4..].trim_start(), words, !excluded, &mut state)
                        {
                            return None;
                        }
                        phase = Phase::InBranch {
                            body_start,
                            excluded,
                        };
                    } else if !self.consume_nested(
                        slice,
                        &cmd.words,
                        !prev_already_taken,
                        &mut state,
                    ) {
                        return None;
                    }
                }
                Phase::InBranch {
                    body_start,
                    excluded,
                } => {
                    if state.depth > 0 {
                        if !self.consume_nested(slice, &cmd.words, !excluded, &mut state) {
                            return None;
                        }
                    } else {
                        match keyword(slice) {
                            "elif" if !saw_else => {
                                if excluded {
                                    pending.push(body_start..cmd_start);
                                }
                                let cond_words = drop_leading_word(&cmd.words, "elif");
                                if !self.consume_body(
                                    slice[4..].trim_start(),
                                    cond_words,
                                    !already_taken,
                                    &mut state,
                                ) {
                                    return None;
                                }
                                phase = Phase::InCondition {
                                    cond_start: cmd_start,
                                    cond_cmd_idx: idx,
                                    prev_already_taken: already_taken,
                                };
                            }
                            "else" if !saw_else => {
                                if excluded {
                                    pending.push(body_start..cmd_start);
                                }
                                saw_else = true;
                                let words = drop_leading_word(&cmd.words, "else");
                                if !self.consume_body(
                                    slice[4..].trim_start(),
                                    words,
                                    !already_taken,
                                    &mut state,
                                ) {
                                    return None;
                                }
                                phase = Phase::InBranch {
                                    body_start: cmd_start,
                                    excluded: already_taken,
                                };
                            }
                            "fi" => {
                                if excluded {
                                    pending.push(body_start..cmd_start);
                                }
                                let tail = &slice[2..];
                                if scan::has_complex_expansion(tail)
                                    || scan::has_arithmetic_command(tail)
                                {
                                    self.mutated_all = true;
                                }
                                self.excluded.extend(pending);
                                return Some(idx + 1);
                            }
                            _ => {
                                if !self.consume_body(slice, &cmd.words, !excluded, &mut state) {
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
            idx += 1;
        }
        None
    }

    fn judge_condition(
        &self,
        cond_start: usize,
        cond_cmd_idx: usize,
        then_idx: usize,
        then_start: usize,
        prev_already_taken: bool,
        already_taken: &mut bool,
    ) -> bool {
        if prev_already_taken {
            return true;
        }
        if then_idx != cond_cmd_idx + 1
            || self.script[cond_start..then_start].contains(['<', '>', '|', '&'])
        {
            return false;
        }
        match eval_condition(
            &self.commands[cond_cmd_idx].words,
            self.env,
            &self.mutated_vars,
            self.mutated_all,
        ) {
            OnMergeGroup::AlwaysFalse => true,
            OnMergeGroup::AlwaysTrue => {
                *already_taken = true;
                false
            }
            OnMergeGroup::Unknown => false,
        }
    }
}
