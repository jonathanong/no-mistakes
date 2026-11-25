use super::{
    super::{super::lex::Boundary, frames::Frame, loop_frame::LoopPass},
    Flow,
};

impl<'a> Flow<'a> {
    pub(super) fn finish_loop(&mut self, line: usize, outgoing: Boundary<'a>, redirected: bool) {
        let Some(Frame::Repeat(repeat)) = self.frames.last_mut() else {
            self.unanalyzable.push((
                line,
                "unmatched `done` without an enclosing loop; cannot soundly prove array safety"
                    .into(),
            ));
            return;
        };
        let exhausted = match repeat.next_pass(&mut self.state, self.loop_replays_left > 0) {
            LoopPass::Replay(restart) => {
                self.loop_replays_left -= 1;
                self.reset_list();
                self.restart = Some(restart);
                return;
            }
            LoopPass::Exhausted => {
                self.unanalyzable.push((
                    line,
                    "file-wide Bash loop replay budget exhausted; cannot prove array safety on later iterations".into(),
                ));
                true
            }
            LoopPass::Stable => false,
        };
        if let Some(Frame::Repeat(repeat)) = self.frames.pop() {
            let leave = repeat.finish(
                &mut self.state,
                outgoing == Boundary::Other || redirected,
                exhausted,
            );
            if leave.is_leaving() {
                self.observe_statement(leave);
            }
        }
    }
}
