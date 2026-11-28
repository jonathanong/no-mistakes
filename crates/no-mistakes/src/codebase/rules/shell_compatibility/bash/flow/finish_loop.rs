use super::{
    super::{
        super::lex::{Boundary, Word},
        frames::Frame,
    },
    Flow,
};
use crate::fx::FxHashMap;

pub(super) fn is_infinite_loop(
    keyword: &str,
    rest: &[&Word<'_>],
    seen_functions: &FxHashMap<&str, usize>,
) -> bool {
    match (keyword, rest) {
        ("while", [word]) => {
            matches!(word.raw, ":" | "true") && !seen_functions.contains_key(word.raw)
        }
        ("until", [word]) => word.raw == "false" && !seen_functions.contains_key("false"),
        _ => false,
    }
}

pub(super) fn for_has_literal_item(rest: &[&Word<'_>]) -> bool {
    rest.get(1).is_some_and(|word| word.raw == "in") && rest[2..].iter().any(|word| !word.dynamic)
}

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
