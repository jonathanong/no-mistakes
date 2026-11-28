use super::{
    at_keyword_position, function_keyword_named, is_coproc_prefix, Boundary, CompoundBody,
    Function, Group, Lexer, ScanState, Word,
};

impl<'a> Lexer<'a> {
    pub(super) fn emit_isolated_arith(
        &mut self,
        func: Option<Function<'a>>,
        word: Word<'a>,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        if !st.words.is_empty() {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        }
        let line = word.line;
        let enters_function = func.is_some();
        st.scope += usize::from(enters_function);
        visit(
            Vec::new(),
            st.scope,
            Boundary::Open {
                isolated: true,
                function: func.and_then(Function::name).map(|name| (name, false)),
                enters_function,
                line,
            },
        );
        visit(vec![word], st.scope, Boundary::End);
        st.scope -= usize::from(enters_function);
        self.emit_close_with_redirs(st.scope, visit);
    }

    pub(super) fn push_bash_word(
        &mut self,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        let was_conditional = st.conditional;
        let in_pattern = st.cases.in_pattern();
        let coproc = !was_conditional && !in_pattern && is_coproc_prefix(&st.words);
        let pending = (!was_conditional && !in_pattern && !coproc)
            .then(|| {
                st.pending_function
                    .take()
                    .or_else(|| function_keyword_named(&st.words))
            })
            .flatten();
        if let Some(func) = pending {
            let word = self.word();
            if let Some(body) = CompoundBody::from_opener(word.raw) {
                self.open_compound(Some(func), body, word, st, visit);
            } else {
                visit(Vec::new(), st.scope, Boundary::Unanalyzable(word.line));
            }
            return;
        }
        let words_empty = st.words.is_empty();
        let keyword_pos = !was_conditional && at_keyword_position(&st.words);
        if coproc {
            let word = self.word();
            if let Some(body) = CompoundBody::from_opener(word.raw) {
                self.open_compound(None, body, word, st, visit);
                return;
            }
            let starts_patterns = self.finish_pushed_word(
                word,
                &mut st.words,
                &mut st.conditional,
                &mut st.cases,
                true,
            );
            if starts_patterns {
                visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
            }
            return;
        }
        let starts_patterns =
            self.push_word(&mut st.words, &mut st.conditional, &mut st.cases, true);
        let pushed_raw = st.words.last().filter(|w| !w.redirect).map(|w| w.raw);
        let mut close_compound = None;
        if let Some(Group::Compound {
            body,
            depth,
            is_function,
        }) = st.groups.last_mut()
        {
            if *body == CompoundBody::Bracket {
                if was_conditional && !st.conditional {
                    close_compound = Some(*is_function);
                }
            } else if keyword_pos && (!in_pattern || (*body == CompoundBody::Case && words_empty)) {
                if pushed_raw.is_some_and(|raw| CompoundBody::from_opener(raw) == Some(*body)) {
                    *depth += 1;
                } else if pushed_raw == Some(body.closer()) {
                    *depth -= 1;
                    if *depth == 0 {
                        close_compound = Some(*is_function);
                    }
                }
            }
        }
        if let Some(is_function) = close_compound {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
            st.groups.pop();
            st.scope -= usize::from(is_function);
            self.emit_close_with_redirs(st.scope, visit);
            return;
        }
        if starts_patterns {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        }
    }

    fn open_compound(
        &mut self,
        func: Option<Function<'a>>,
        body: CompoundBody,
        word: Word<'a>,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        if !st.words.is_empty() {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        }
        let line = word.line;
        let is_function = func.is_some();
        st.scope += usize::from(is_function);
        st.groups.push(Group::Compound {
            body,
            depth: 1,
            is_function,
        });
        visit(
            Vec::new(),
            st.scope,
            Boundary::Open {
                isolated: true,
                function: func.and_then(Function::name).map(|name| (name, false)),
                enters_function: is_function,
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
    }
}
