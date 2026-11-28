use super::{
    super::{frames::Frame, guard::Leave},
    Flow,
};

impl Flow<'_> {
    pub(super) fn observe_statement(&mut self, leave: Leave) {
        match self.frames.last_mut() {
            Some(Frame::If(branch)) => branch.command(leave),
            Some(Frame::Case(case)) => case.command(leave),
            Some(Frame::Repeat(repeat)) => repeat.command(leave),
            Some(Frame::Group(group)) => group.leave = group.leave.combine(leave),
            None => {}
        }
    }
}
