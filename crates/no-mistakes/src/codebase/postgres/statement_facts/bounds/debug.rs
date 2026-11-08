use super::SqlBoundFact;
use std::fmt::{Debug, Formatter, Result};

impl Debug for SqlBoundFact {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut out = f.debug_struct("SqlBoundFact");
        out.field("kind", &self.kind);
        if let Some(body) = &self.body {
            out.field("body", body);
        }
        out.field("line", &self.line)
            .field("column", &self.column)
            .field("statement_start", &self.statement_start)
            .field("query", &self.query)
            .field("target", &self.target)
            .finish()
    }
}
