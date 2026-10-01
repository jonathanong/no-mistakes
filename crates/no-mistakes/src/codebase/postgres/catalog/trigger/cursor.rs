use super::super::model::{TriggerEvent, TriggerTiming};

pub(super) struct Cursor<'a> {
    pub(super) source: &'a str,
    pub(super) index: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }

    pub(super) fn done(&self) -> bool {
        self.index >= self.source.len()
    }

    pub(super) fn rest(&self) -> &'a str {
        &self.source[self.index..]
    }

    pub(super) fn skip_ws(&mut self) {
        self.index += self.rest().len() - self.rest().trim_start().len();
    }

    pub(super) fn eat_char(&mut self, expected: char) -> bool {
        self.skip_ws();
        if self.rest().starts_with(expected) {
            self.index += expected.len_utf8();
            true
        } else {
            false
        }
    }

    pub(super) fn expect_char(&mut self, expected: char) -> Result<(), String> {
        if self.eat_char(expected) {
            Ok(())
        } else {
            Err(format!("expected {expected}"))
        }
    }

    pub(super) fn eat_kw(&mut self, word: &str) -> bool {
        self.skip_ws();
        let rest = self.rest();
        let Some(slice) = rest.get(..word.len()) else {
            return false;
        };
        if !slice.eq_ignore_ascii_case(word) {
            return false;
        }
        let after = rest.as_bytes().get(word.len());
        if after.is_some_and(is_ident_byte) {
            return false;
        }
        self.index += word.len();
        true
    }

    pub(super) fn expect_kw(&mut self, word: &str) -> Result<(), String> {
        if self.eat_kw(word) {
            Ok(())
        } else {
            Err(format!("expected {word}"))
        }
    }

    pub(super) fn take_timing(&mut self) -> Result<TriggerTiming, String> {
        if self.eat_kw("before") {
            Ok(TriggerTiming::Before)
        } else if self.eat_kw("after") {
            Ok(TriggerTiming::After)
        } else if self.eat_kw("instead") {
            self.expect_kw("of")?;
            Ok(TriggerTiming::InsteadOf)
        } else {
            Err("expected trigger timing".to_string())
        }
    }

    pub(super) fn take_events(&mut self) -> Result<(Vec<TriggerEvent>, Vec<String>), String> {
        let mut events = Vec::new();
        let mut update_columns = Vec::new();
        loop {
            if self.eat_kw("insert") {
                events.push(TriggerEvent::Insert);
            } else if self.eat_kw("delete") {
                events.push(TriggerEvent::Delete);
            } else if self.eat_kw("truncate") {
                events.push(TriggerEvent::Truncate);
            } else if self.eat_kw("update") {
                events.push(TriggerEvent::Update);
                if self.eat_kw("of") {
                    loop {
                        update_columns.push(self.take_ident()?);
                        if !self.eat_char(',') {
                            break;
                        }
                    }
                }
            } else {
                return Err("expected trigger event".to_string());
            }
            if !self.eat_kw("or") {
                break;
            }
        }
        Ok((events, update_columns))
    }

    pub(super) fn skip_constraint_clauses(&mut self) -> Result<(), String> {
        if self.eat_kw("from") {
            self.take_qualified()?;
        }
        if self.eat_kw("not") {
            self.expect_kw("deferrable")?;
        } else {
            let _ = self.eat_kw("deferrable");
        }
        if self.eat_kw("initially") && !(self.eat_kw("immediate") || self.eat_kw("deferred")) {
            return Err("expected immediate or deferred".to_string());
        }
        if self.eat_kw("referencing") {
            while self.eat_kw("old") || self.eat_kw("new") {
                self.expect_kw("table")?;
                let _ = self.eat_kw("as");
                self.take_ident()?;
            }
        }
        Ok(())
    }
}

pub(super) fn is_ident_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

pub(super) fn is_ident_char(character: char) -> bool {
    is_ident_start(character) || character.is_ascii_digit() || character == '$'
}

fn is_ident_byte(byte: &u8) -> bool {
    byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'$'
}
