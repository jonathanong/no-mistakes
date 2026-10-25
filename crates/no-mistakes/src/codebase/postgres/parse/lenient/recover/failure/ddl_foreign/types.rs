use super::cursor::Cursor;

impl Cursor<'_> {
    pub(super) fn type_name(&mut self) -> bool {
        let start = self.at;
        let temporal = self.word("TIMESTAMP") || self.word("TIME");
        if temporal {
            if !self.type_precision(false) {
                return false;
            }
            if (self.word("WITH") || self.word("WITHOUT"))
                && !(self.word("TIME") && self.word("ZONE"))
            {
                return false;
            }
        } else {
            // A failed built-in must not let a later token masquerade as its type.
            if !self.type_keyword() && (self.at != start || !self.name()) {
                return false;
            }
            if !self.type_precision(true) {
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

    fn type_precision(&mut self, scale: bool) -> bool {
        if !self.left_paren() {
            return true;
        }
        self.number() && (!scale || !self.comma() || self.number()) && self.right_paren()
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
        [
            "INT", "INTEGER", "TEXT", "BOOLEAN", "DATE", "INTERVAL", "NUMERIC", "DECIMAL", "REAL",
            "SMALLINT", "BIGINT",
        ]
        .iter()
        .any(|value| self.word(value))
    }
}
