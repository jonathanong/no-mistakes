use super::{
    super::{
        super::lex::{Boundary, Word},
        branch_frame::Branch,
        case_frame::Case,
        case_unset::subject_vars_from_header,
        frames::Frame,
        guard::{Guard, Leave},
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
}
