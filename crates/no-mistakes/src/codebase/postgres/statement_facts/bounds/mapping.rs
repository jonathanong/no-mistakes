use super::{SqlBoundFact, SqlBoundItemKind, SqlBoundQuery, SqlPinSource};

impl SqlBoundFact {
    /// Map every position through `map(line, column)`, for SQL embedded in a host file: the
    /// column decides which physical operand of a recovered string owns the line.
    pub fn map_lines(&mut self, map: &impl Fn(usize, usize) -> usize) {
        self.line = map(self.line, self.column);
        if let Some((line, column)) = &mut self.statement_start {
            *line = map(*line, *column);
        }
        self.query.map_lines(map);
    }
}

impl SqlBoundQuery {
    // Borrow one callback through the recursive tree instead of specializing each traversal.
    fn map_lines(&mut self, map: &dyn Fn(usize, usize) -> usize) {
        for item in &mut self.items {
            item.line = map(item.line, item.column);
            if let SqlBoundItemKind::Query(query) = &mut item.kind {
                query.map_lines(map);
            }
            for pin in &mut item.pins {
                if let SqlPinSource::Query(query)
                | SqlPinSource::ReadQuery(query)
                | SqlPinSource::ArrayQuery(query) = &mut pin.source
                {
                    query.map_lines(map);
                }
            }
        }
    }
}
