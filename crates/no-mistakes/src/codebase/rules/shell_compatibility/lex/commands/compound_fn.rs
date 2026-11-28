use super::{
    at_keyword_position, function_keyword_named, Boundary, CompoundBody, Function, Group, Lexer,
    ScanState, Word,
};

impl<'a> Lexer<'a> {
    pub(super) fn emit_arith_function(
        &mut self,
        func: Function<'a>,
        word: Word<'a>,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        if !st.words.is_empty() {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        }
        let line = word.line;
        st.scope += 1;
        visit(
            Vec::new(),
            st.scope,
            Boundary::Open {
                isolated: true,
                function: func.name().map(|name| (name, false)),
                enters_function: true,
                line,
            },
        );
        visit(vec![word], st.scope, Boundary::End);
        st.scope -= 1;
        self.emit_close_with_redirs(st.scope, visit);
    }

    pub(super) fn push_bash_word(
        &mut self,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        let was_conditional = st.conditional;
        let in_pattern = st.cases.in_pattern();
        let pending = (!was_conditional && !in_pattern)
            .then(|| {
                st.pending_function
                    .take()
                    .or_else(|| function_keyword_named(&st.words))
            })
            .flatten();
        if let Some(func) = pending {
            let word = self.word();
            if let Some(body) = CompoundBody::from_opener(word.raw) {
                if !st.words.is_empty() {
                    visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
                }
                let line = word.line;
                st.scope += 1;
                st.groups.push(Group::CompoundFunction { body, depth: 1 });
                visit(
                    Vec::new(),
                    st.scope,
                    Boundary::Open {
                        isolated: true,
                        function: func.name().map(|name| (name, false)),
                        enters_function: true,
                        line,
                    },
                );
                self.finish_pushed_word(
                    word,
                    &mut st.words,
                    &mut st.conditional,
                    &mut st.cases,
                    true,
                );
            } else {
                visit(Vec::new(), st.scope, Boundary::Unanalyzable(word.line));
            }
            return;
        }
        let words_empty = st.words.is_empty();
        let keyword_pos = !was_conditional && at_keyword_position(&st.words);
        let starts_patterns =
            self.push_word(&mut st.words, &mut st.conditional, &mut st.cases, true);
        let pushed_raw = st.words.last().filter(|w| !w.redirect).map(|w| w.raw);
        let mut close_compound = false;
        if let Some(Group::CompoundFunction { body, depth }) = st.groups.last_mut() {
            if *body == CompoundBody::Bracket {
                close_compound = was_conditional && !st.conditional;
            } else if keyword_pos && (!in_pattern || (*body == CompoundBody::Case && words_empty)) {
                if pushed_raw.is_some_and(|raw| CompoundBody::from_opener(raw) == Some(*body)) {
                    *depth += 1;
                } else if pushed_raw == Some(body.closer()) {
                    *depth -= 1;
                    close_compound = *depth == 0;
                }
            }
        }
        if close_compound {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
            st.groups.pop();
            st.scope -= 1;
            self.emit_close_with_redirs(st.scope, visit);
            return;
        }
        if starts_patterns {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        }
    }
}
