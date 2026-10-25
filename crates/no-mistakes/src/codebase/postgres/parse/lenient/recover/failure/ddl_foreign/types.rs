use super::cursor::Cursor;

impl Cursor<'_> {
    pub(super) fn type_name(&mut self) -> bool {
        if !(self.type_keyword() || self.name()) {
            return false;
        }
        if self.left_paren() {
            if !self.number() {
                return false;
            }
            if self.comma() && !self.number() {
                return false;
            }
            if !self.right_paren() {
                return false;
            }
        }
        if self.left_bracket() && !self.right_bracket() {
            return false;
        }
        true
    }
    pub(super) fn type_keyword(&mut self) -> bool {
        if self.word("DOUBLE") {
            return self.word("PRECISION");
        }
        if self.word("CHARACTER") || self.word("CHAR") {
            self.word("VARYING");
            return true;
        }
        if self.word("VARCHAR") {
            return true;
        }
        if self.word("TIMESTAMP") || self.word("TIME") {
            if self.word("WITH") || self.word("WITHOUT") {
                return self.word("TIME") && self.word("ZONE");
            }
            return true;
        }
        [
            "INT", "INTEGER", "TEXT", "BOOLEAN", "DATE", "INTERVAL", "NUMERIC", "DECIMAL", "REAL",
            "SMALLINT", "BIGINT",
        ]
        .iter()
        .any(|value| self.word(value))
    }
}
