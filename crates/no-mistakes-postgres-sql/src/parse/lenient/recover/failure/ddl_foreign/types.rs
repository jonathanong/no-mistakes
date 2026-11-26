use super::cursor::Cursor;
use sqlparser::tokenizer::Token;

impl Cursor<'_> {
    pub(super) fn type_name(&mut self) -> bool {
        // Qualified names use PostgreSQL's function/type identifier class.
        let qualified = self
            .tokens
            .get(self.at + 1)
            .is_some_and(|t| **t == Token::Period);
        if !qualified && (self.word("TIMESTAMP") || self.word("TIME")) {
            if !self.type_precision(1) {
                return false;
            }
            if (self.word("WITH") || self.word("WITHOUT"))
                && !(self.word("TIME") && self.word("ZONE"))
            {
                return false;
            }
        } else {
            let width = if qualified {
                None
            } else {
                self.builtin_width()
            };
            let width = match width {
                Some(width) => width,
                None if self.function_name() => 2,
                None => return false,
            };
            if !self.type_precision(width) {
                return false;
            }
        }
        while self.left_bracket() {
            self.number();
            if !self.right_bracket() {
                return false;
            }
        }
        true
    }

    fn type_precision(&mut self, width: usize) -> bool {
        if !self.left_paren() {
            return true;
        }
        width > 0
            && self.number()
            && (width < 2 || !self.comma() || self.number())
            && self.right_paren()
    }

    fn builtin_width(&mut self) -> Option<usize> {
        if self.word("DOUBLE") {
            // Bare double is a valid custom type name, not an incomplete builtin.
            return Some(if self.word("PRECISION") { 0 } else { 2 });
        }
        if self.word("CHARACTER") || self.word("CHAR") {
            self.word("VARYING");
            return Some(1);
        }
        if self.word("VARCHAR") || self.word("FLOAT") || self.word("INTERVAL") {
            return Some(1);
        }
        if self.word("NUMERIC") || self.word("DECIMAL") || self.word("DEC") {
            return Some(2);
        }
        [
            "INT", "INTEGER", "TEXT", "BOOLEAN", "DATE", "REAL", "SMALLINT", "BIGINT", "INT2",
            "INT4", "INT8", "BOOL", "FLOAT4", "FLOAT8", "BYTEA",
        ]
        .iter()
        .any(|value| self.word(value))
        .then_some(0)
    }
}
