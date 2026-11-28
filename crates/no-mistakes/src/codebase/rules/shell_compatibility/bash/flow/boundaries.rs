use super::{
    super::{frames::Group, guard::Leave, state::ArrayState, write_sets::WriteSets},
    Boundary, Flow, Frame,
};
use crate::fx::FxHashMap;

impl<'a> Flow<'a> {
    pub(super) fn handle_boundary(
        &mut self,
        index: usize,
        outgoing: Boundary<'a>,
        empty: bool,
        redirected: bool,
        write_sets: &WriteSets,
        analyze: &mut impl FnMut(&mut super::State, &mut Vec<Frame<'a>>),
    ) -> bool {
        use Boundary::{And, End, Or, Other};
        match outgoing {
            Boundary::Unanalyzable(line) => {
                self.unanalyzable.push((
                    line,
                    "unmodeled inline subshell or group syntax; rewrite the guard as a top-level test or guard the expansion with ${name[@]+...}".into(),
                ));
                self.reset_list();
            }
            Boundary::Open {
                isolated,
                function,
                enters_function,
                line,
            } => {
                if let Some(Frame::If(branch)) = self.frames.last_mut() {
                    branch.invalidate_condition();
                }
                self.observe_statement(Leave::None);
                let brace_function = if let Some((name, brace_body)) = function {
                    if matches!(name, ":" | "true" | "false") {
                        for frame in &mut self.frames {
                            if let Frame::Repeat(repeat) = frame {
                                repeat.invalidate_infinite();
                            }
                        }
                    }
                    let first = *self.seen_functions.entry(name).or_insert(index);
                    if first != index {
                        self.redefined_functions.insert(name);
                        self.clear_function(name);
                        self.unanalyzable.push((
                            line,
                            format!(
                                "function `{name}` is redefined; cannot soundly prove array safety across redefinitions"
                            ),
                        ));
                    }
                    brace_body.then_some(name)
                } else {
                    None
                };
                let before = self.state.clone();
                if enters_function {
                    self.state.nounset_effect = None;
                    self.state.clear_current_locals();
                    self.state.forget_nonempty_for_function(index, write_sets);
                } else if matches!(self.incoming, And | Or) {
                    if let Some(guard) = self.pending {
                        self.state.set_known(guard.name, ArrayState::Empty);
                    }
                    self.state.mark_guarded_maybe();
                }
                self.frames.push(Frame::Group(Box::new(Group {
                    open_index: index,
                    before,
                    isolated,
                    optional: self.incoming != End,
                    function: brace_function,
                    enters_function,
                    guard: self.pending,
                    leave: Leave::None,
                    return_true: None,
                    return_nounset: None,
                    saved_fns: None,
                    unset_fns: FxHashMap::default(),
                    dynamic_fn_unset: false,
                })));
                self.reset_list();
            }
            Boundary::Close { completes_list } => {
                if let Some(Frame::Group(mut group)) = self.frames.pop() {
                    if !empty {
                        if group.enters_function {
                            let mut redir = group.before.clone();
                            redir.clear_current_locals();
                            redir.forget_nonempty_for_function(group.open_index, write_sets);
                            analyze(&mut redir, &mut self.frames);
                        } else {
                            analyze(&mut group.before, &mut self.frames);
                        }
                    }
                    if let Some(saved) = group.saved_fns.take() {
                        self.restore_fns(saved);
                    }
                    if let Some(name) = group.function {
                        if self.frames.is_empty() && !self.redefined_functions.contains(name) {
                            self.record_function_summary(name, &group, redirected, write_sets);
                        }
                    }
                    let leave = group.finish(&mut self.state, completes_list, redirected);
                    if leave.is_leaving() {
                        self.observe_statement(leave);
                    }
                }
                self.reset_list();
            }
            Boundary::CaseArm(kind) if empty => {
                if let Some(Frame::Case(case)) = self.frames.last_mut() {
                    case.arm_end(kind, &mut self.state);
                }
                self.reset_list();
            }
            _ if empty => {
                if matches!(outgoing, And | Or | Other) {
                    self.incoming = outgoing;
                    self.pending = None;
                    self.pending_cond = None;
                }
            }
            _ => return false,
        }
        true
    }

    pub(super) fn reset_list(&mut self) {
        self.incoming = Boundary::End;
        self.pending = None;
        self.pending_cond = None;
    }

    pub(super) fn record_function_return(&mut self) {
        for frame in self.frames.iter_mut().rev() {
            if let Frame::Group(group) = frame {
                if group.function.is_some() {
                    group.record_return(&self.state);
                    return;
                }
                if group.isolated {
                    return;
                }
            }
        }
    }

    pub(super) fn record_loop_jump(&mut self, levels: usize, is_break: bool) {
        if levels == 0 {
            return;
        }
        let mut remaining = levels;
        let mut target = None;
        for (index, frame) in self.frames.iter().enumerate().rev() {
            match frame {
                Frame::Repeat(_) => {
                    target = Some(index);
                    remaining -= 1;
                    if remaining == 0 {
                        break;
                    }
                }
                Frame::Group(group) if group.isolated => break,
                _ => {}
            }
        }
        let Some(index) = target else {
            return;
        };
        if let Frame::Repeat(repeat) = &mut self.frames[index] {
            if is_break {
                repeat.record_break(&self.state);
            } else {
                repeat.record_continue(&self.state);
            }
        }
    }
}
