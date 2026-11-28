use super::{
    closes, function_keyword, group_start, Boundary, Function, Group, Lexer, ScanState, Word,
};

impl<'a> Lexer<'a> {
    pub(super) fn can_open_group(&self, byte: u8, st: &ScanState<'a>) -> bool {
        (byte == b'{' || st.is_bash)
            && !(st.is_bash && st.cases.in_pattern())
            && group_start(&st.words)
            && (byte == b'(' || self.brace_boundary())
    }

    pub(super) fn can_close_group(&self, byte: u8, st: &ScanState<'a>) -> bool {
        closes(st.groups.last(), byte)
            && !(st.is_bash && st.cases.in_pattern())
            && (byte == b')' || (group_start(&st.words) && self.brace_boundary()))
    }

    pub(super) fn open_group(
        &mut self,
        byte: u8,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        let function = st
            .pending_function
            .take()
            .or_else(|| function_keyword(&st.words));
        let group = match (function.is_some(), byte) {
            (true, _) => Group::Function(byte == b'{'),
            (false, b'{') => Group::Brace,
            (false, _) => Group::Subshell,
        };
        let line = self.line;
        visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        st.scope += usize::from(function.is_some());
        st.groups.push(group);
        self.advance();
        visit(
            Vec::new(),
            st.scope,
            Boundary::Open {
                isolated: !matches!(group, Group::Brace),
                function: function
                    .and_then(Function::name)
                    .map(|name| (name, byte == b'{')),
                enters_function: function.is_some(),
                line,
            },
        );
    }

    pub(super) fn close_group(
        &mut self,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
        st.scope -= usize::from(matches!(st.groups.pop(), Some(Group::Function(_))));
        self.advance();
        self.emit_close_with_redirs(st.scope, visit);
    }

    pub(super) fn unmodeled_subshell(
        &mut self,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        let line = self.line;
        visit(
            std::mem::take(&mut st.words),
            st.scope,
            Boundary::Unanalyzable(line),
        );
        st.groups.push(Group::Subshell);
        self.advance();
        visit(
            Vec::new(),
            st.scope,
            Boundary::Open {
                isolated: true,
                function: None,
                enters_function: false,
                line,
            },
        );
    }

    pub(super) fn handle_separator(
        &mut self,
        byte: u8,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        if !st.is_bash && byte == b'(' && self.take("((") {
            visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
            self.skip_group(b')');
            self.take(")");
            return;
        }
        let boundary = self.command_separator(
            byte,
            &st.words,
            &mut st.pending_function,
            &mut st.cases,
            st.is_bash,
        );
        visit(std::mem::take(&mut st.words), st.scope, boundary);
        if st.is_bash && matches!(boundary, Boundary::CaseArm(_)) {
            visit(Vec::new(), st.scope, boundary);
        }
    }

    pub(super) fn emit_close_with_redirs(
        &mut self,
        scope: usize,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        let completes_list = self.completes_list();
        let mut words = Vec::new();
        self.take_trailing_redirections(&mut words);
        visit(words, scope, Boundary::Close { completes_list });
    }
}
