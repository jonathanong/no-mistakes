use super::{
    super::{super::lex::Boundary, frames::Frame},
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
        if let Some(restart) = repeat.next_pass(&mut self.state) {
            self.reset_list();
            self.restart = Some(restart);
            return;
        }
        if let Some(Frame::Repeat(repeat)) = self.frames.pop() {
            let leave = repeat.finish(&mut self.state, outgoing == Boundary::Other || redirected);
            if leave.is_leaving() {
                self.observe_statement(leave);
            }
        }
    }
}
