use super::{
    super::{
        super::lex::{identifier, Boundary, Word},
        branch_frame::Branch,
        case_frame::Case,
        case_unset::{subject_vars_from_header, CaseUnsetCtx},
        frames::Frame,
        guard::{Guard, Leave},
        loop_frame::{for_has_literal_item, is_infinite_loop, LoopKind, Repeat},
        write_sets::WriteSets,
    },
    Flow,
};

impl<'a> Flow<'a> {
    pub(super) fn prefix<'w, 's>(
        &mut self,
        index: usize,
        mut words: &'s [&'w Word<'a>],
        outgoing: Boundary<'a>,
        redirected: bool,
        write_sets: &'a WriteSets,
        hits_len: usize,
    ) -> (&'s [&'w Word<'a>], bool) {
        let mut compound = false;
        while let Some((first, rest)) = words.split_first() {
            match first.raw {
                "!" if rest.first().is_some_and(|word| word.raw == "case") => {}
                "then" => {
                    if let Some(Frame::If(branch)) = self.frames.last_mut() {
                        branch.then(&mut self.state);
                    }
                }
                "elif" | "else" => {
                    if let Some(Frame::If(branch)) = self.frames.last_mut() {
                        branch.next_branch(&mut self.state, first.raw == "else");
                    }
                    compound = first.raw == "elif";
                }
                "do" => {
                    if let Some(Frame::Repeat(repeat)) = self.frames.last_mut() {
                        repeat.enter_body(
                            index,
                            (hits_len, self.unanalyzable.len()),
                            &mut self.state,
                        );
                    }
                }
                "if" => {
                    self.enter_if(index, rest, outgoing, redirected);
                    return (rest, true);
                }
                "fi" => {
                    self.finish_if(index, outgoing, redirected, write_sets);
                    return (&[], true);
                }
                "case" => {
                    self.observe_statement(Leave::None);
                    let subject = subject_vars_from_header(rest, &self.state.derived_from);
                    self.frames.push(Frame::Case(Box::new(Case::new(
                        self.state.clone(),
                        self.incoming != Boundary::End,
                        subject,
                    ))));
                    return (rest, true);
                }
                "esac" => {
                    if let Some(Frame::Case(case)) = self.frames.pop() {
                        let leave =
                            case.finish(&mut self.state, outgoing == Boundary::Other || redirected);
                        if leave.is_leaving() {
                            self.observe_statement(leave);
                        }
                    }
                    return (&[], true);
                }
                "for" | "while" | "until" | "select" => {
                    self.enter_loop(
                        (index, hits_len),
                        (first.raw, rest),
                        outgoing,
                        redirected,
                        write_sets,
                    );
                    return (rest, true);
                }
                "done" => {
                    self.finish_loop(first.line, outgoing, redirected);
                    return (&[], true);
                }
                _ => break,
            }
            self.reset_list();
            words = rest;
        }
        (words, compound)
    }

    fn enter_if(
        &mut self,
        index: usize,
        rest: &[&Word<'a>],
        outgoing: Boundary<'a>,
        redirected: bool,
    ) {
        self.observe_statement(Leave::None);
        let guard = (!redirected && outgoing == Boundary::End)
            .then(|| Guard::parse(rest))
            .flatten();
        self.frames.push(Frame::If(Box::new(Branch::new(
            index,
            self.state.clone(),
            guard,
            self.incoming != Boundary::End,
        ))));
    }

    fn finish_if(
        &mut self,
        index: usize,
        outgoing: Boundary<'a>,
        redirected: bool,
        write_sets: &'a WriteSets,
    ) {
        if let Some(Frame::If(branch)) = self.frames.pop() {
            let leave = branch.finish(
                index,
                &mut self.state,
                outgoing == Boundary::Other || redirected,
                write_sets,
            );
            if leave.is_leaving() {
                self.observe_statement(leave);
            }
        }
    }

    fn enter_loop(
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
}
