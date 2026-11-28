use super::{command_words, identifier, Lexer, Shell, Word};
use cases::Cases;
use groups::{
    at_keyword_position, closes, function_keyword, function_keyword_named, function_parentheses,
    group_start, CompoundBody, Function, Group,
};

mod cases;
mod compound_fn;
mod dispatch;
mod groups;
mod tokens;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum CaseArm {
    Break,
    Fallthrough,
    TestNext,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum Boundary<'a> {
    End,
    And,
    Or,
    Other,
    CaseArm(CaseArm),
    CasePattern,
    Unanalyzable(usize),
    Open {
        isolated: bool,
        function: Option<(&'a str, bool)>,
        enters_function: bool,
        line: usize,
    },
    Close {
        completes_list: bool,
    },
}

struct ScanState<'a> {
    words: Vec<Word<'a>>,
    scope: usize,
    groups: Vec<Group>,
    pending_function: Option<Function<'a>>,
    conditional: bool,
    cases: Cases,
    is_bash: bool,
}

impl<'a> Lexer<'a> {
    pub(in super::super) fn commands(
        mut self,
        visit: impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        self.visit_commands(false, visit);
    }

    pub(super) fn visit_commands(
        &mut self,
        substitution: bool,
        mut visit: impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) -> bool {
        let mut st = ScanState {
            words: Vec::new(),
            scope: 0,
            groups: Vec::new(),
            pending_function: None,
            conditional: false,
            cases: Cases::default(),
            is_bash: matches!(self.shell, Shell::Bash),
        };
        while let Some(byte) = self.current() {
            if self.continuation() || self.try_arithmetic(byte, &mut st, &mut visit) {
                continue;
            }
            if byte == b')'
                && substitution
                && st.groups.is_empty()
                && !st.conditional
                && !(st.is_bash && st.cases.in_pattern())
            {
                self.advance();
                return true;
            }
            self.step(byte, &mut st, &mut visit);
        }
        visit(st.words, st.scope, Boundary::End);
        false
    }

    fn try_arithmetic(
        &mut self,
        byte: u8,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) -> bool {
        if st.is_bash
            && byte == b'('
            && !st.conditional
            && !st.cases.in_pattern()
            && self.rest().starts_with("((")
        {
            let for_arith = {
                let mut cmd = command_words(&st.words);
                cmd.next().is_some_and(|w| w.raw == "for") && cmd.next().is_none()
            };
            if let Some(word) = self.arithmetic_word(for_arith) {
                if !for_arith {
                    let pending = st
                        .pending_function
                        .take()
                        .or_else(|| function_keyword_named(&st.words));
                    if let Some(func) = pending {
                        self.emit_arith_function(func, word, st, visit);
                        return true;
                    }
                }
                st.words.push(word);
                if for_arith {
                    visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
                }
                return true;
            }
        }
        false
    }

    fn step(
        &mut self,
        byte: u8,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) {
        if self.step_special(byte, st, visit) {
            return;
        }
        match byte {
            b'&' if self.rest().starts_with("&>") => st.words.extend(self.redirection()),
            b'(' if self.rest().starts_with("()")
                && function_parentheses(&st.words, !st.is_bash).is_some() =>
            {
                st.pending_function = function_parentheses(&st.words, !st.is_bash);
                self.take("()");
                visit(std::mem::take(&mut st.words), st.scope, Boundary::End);
            }
            b'{' | b'(' if self.can_open_group(byte, st) => self.open_group(byte, st, visit),
            b'}' | b')' if self.can_close_group(byte, st) => self.close_group(st, visit),
            b'(' if st.is_bash => self.unmodeled_subshell(st, visit),
            b'\n' | b';' | b'|' | b'&' | b'(' | b')' => self.handle_separator(byte, st, visit),
            b'<' | b'>' => st.words.extend(self.redirection()),
            _ => {
                if st.is_bash {
                    self.push_bash_word(st, visit);
                } else {
                    st.pending_function = None;
                    let _ =
                        self.push_word(&mut st.words, &mut st.conditional, &mut st.cases, false);
                }
            }
        }
    }

    fn step_special(
        &mut self,
        byte: u8,
        st: &mut ScanState<'a>,
        visit: &mut impl FnMut(Vec<Word<'a>>, usize, Boundary<'a>),
    ) -> bool {
        match byte {
            b' ' | b'\t' | b'\r' | b'\x0c' => self.advance(),
            b'#' => self.skip_comment(),
            b'\n' | b';' | b'|' | b'&' | b'(' | b')' | b'<' | b'>' if st.conditional => {
                if byte == b'\n' {
                    self.advance();
                } else {
                    st.words.push(self.separator_word(byte));
                }
            }
            b'(' | b')' | b'|' if st.is_bash && st.cases.in_pattern() => {
                if self.case_punctuation(byte, &mut st.cases, &mut st.words) {
                    visit(
                        std::mem::take(&mut st.words),
                        st.scope,
                        Boundary::CasePattern,
                    );
                }
            }
            _ => return false,
        }
        true
    }
}
