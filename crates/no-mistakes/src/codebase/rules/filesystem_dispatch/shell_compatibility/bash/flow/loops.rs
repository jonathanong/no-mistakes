use super::{
    super::{
        super::lex::{identifier, Boundary, Word},
        case_unset::CaseUnsetCtx,
        frames::Frame,
        guard::Leave,
        loop_frame::{for_has_literal_item, is_infinite_loop, LoopKind, LoopPass, Repeat},
        write_sets::WriteSets,
    },
    Flow,
};

const MAX_NEST_REPLAYS: u8 = 64;

impl<'a> Flow<'a> {
    pub(super) fn enter_loop(
        &mut self,
        (index, hits_len): (usize, usize),
        (keyword, rest): (&str, &[&Word<'a>]),
        outgoing: Boundary<'a>,
        redirected: bool,
        write_sets: &'a WriteSets,
    ) {
        if let Some(Frame::Repeat(repeat)) = self.frames.last() {
            if repeat.header_index == index && repeat.pass > 0 && !repeat.in_body {
                return;
            }
        }
        if self
            .loop_nest_frame
            .is_none_or(|index| !matches!(self.frames.get(index), Some(Frame::Repeat(_))))
        {
            self.loop_nest_frame = Some(self.frames.len());
            self.loop_replays_left = MAX_NEST_REPLAYS;
        }
        self.observe_statement(Leave::None);
        let loop_var = matches!(keyword, "for" | "select")
            .then(|| {
                rest.first()
                    .filter(|word| identifier(word.raw))
                    .map(|w| w.raw)
            })
            .flatten();
        if let Some(var) = loop_var {
            self.state.touch(var);
            self.frames.invalidate(var);
        }
        let infinite = !redirected
            && outgoing == Boundary::End
            && is_infinite_loop(keyword, rest, &self.seen_functions);
        let always_enters = infinite || (keyword == "for" && for_has_literal_item(rest));
        let kind = LoopKind {
            always_enters,
            infinite,
            restart_at_header: matches!(keyword, "while" | "until"),
            optional: self.incoming != Boundary::End,
        };
        self.frames.push(Frame::Repeat(Box::new(Repeat::new(
            (index, hits_len, self.unanalyzable.len()),
            self.state.clone(),
            kind,
            loop_var,
            write_sets.empties(index),
            write_sets.touches(index),
        ))));
    }

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
                self.unanalyzable.push((line, "Bash loop nest replay budget exhausted; cannot prove array safety on later iterations".into()));
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
