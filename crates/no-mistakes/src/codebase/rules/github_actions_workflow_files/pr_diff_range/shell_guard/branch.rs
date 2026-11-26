use super::control::keyword;
use super::eval::{eval_condition, OnMergeGroup};
use super::scan;
use super::{BranchState, GuardScan};

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
        if self.has_arithmetic_at(start_idx) || !self.consume_after(start_idx, "if", true, &mut state)
        {
            return None;
        }
        let mut pending = Vec::new();
        let mut idx = start_idx + 1;
        while idx < self.commands.len() {
            let cmd = &self.commands[idx];
            let cmd_start = self.positions[cmd.anchor];
            let slice = self.slice(idx);
            let closing_outer = matches!(phase, Phase::InBranch { .. })
                && state.depth == 0
                && keyword(slice) == "fi";
            if !closing_outer && self.has_arithmetic_at(idx) {
                return None;
            }
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
                        if !self.consume_after(idx, "then", !excluded, &mut state) {
                            return None;
                        }
                        phase = Phase::InBranch {
                            body_start,
                            excluded,
                        };
                    } else if !self.consume_nested(idx, !prev_already_taken, &mut state) {
                        return None;
                    }
                }
                Phase::InBranch {
                    body_start,
                    excluded,
                } => {
                    if state.depth > 0 {
                        if !self.consume_nested(idx, !excluded, &mut state) {
                            return None;
                        }
                    } else {
                        match keyword(slice) {
                            "elif" if !saw_else => {
                                if excluded {
                                    pending.push(body_start..cmd_start);
                                }
                                if !self.consume_after(idx, "elif", !already_taken, &mut state) {
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
                                if !self.consume_after(idx, "else", !already_taken, &mut state) {
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
                                let range = self.slice_range(idx);
                                let fi_end = range.end - slice.len() + 2;
                                if self.has_complex_expansion(fi_end..range.end)
                                    || scan::has_arithmetic_in_range(
                                        self.script,
                                        fi_end..range.end,
                                        &self.heredocs,
                                    )
                                {
                                    self.mutated_all = true;
                                }
                                self.excluded.extend(pending);
                                return Some(idx + 1);
                            }
                            _ => {
                                let range = self.slice_range(idx);
                                if !self.consume_body(range, &cmd.words, !excluded, &mut state) {
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
