use sqlparser::tokenizer::Location;

pub(super) struct SourceLines<'a> {
    sql: &'a str,
    starts: Vec<usize>,
    unicode_columns: Vec<Option<Vec<usize>>>,
}

impl<'a> SourceLines<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(sql.match_indices('\n').map(|(at, _)| at + 1));
        let unicode_columns = starts
            .iter()
            .enumerate()
            .map(|(line, start)| {
                let end = starts.get(line + 1).copied().unwrap_or(sql.len());
                let text = &sql[*start..end];
                (!text.is_ascii()).then(|| text.char_indices().map(|(at, _)| at).collect())
            })
            .collect();
        Self {
            sql,
            starts,
            unicode_columns,
        }
    }

    pub(super) fn offset(&self, location: Location) -> Option<usize> {
        let line = location.line.checked_sub(1)? as usize;
        let start = *self.starts.get(line)?;
        let end = self.starts.get(line + 1).copied().unwrap_or(self.sql.len());
        let column = location.column.saturating_sub(1) as usize;
        let byte = self.unicode_columns[line]
            .as_ref()
            .map_or(column.min(end - start), |columns| {
                columns.get(column).copied().unwrap_or(end - start)
            });
        Some(start + byte)
    }

    pub(super) fn location(&self, byte: usize) -> Option<Location> {
        let at = self
            .starts
            .partition_point(|start| *start <= byte)
            .checked_sub(1)?;
        let start = self.starts[at];
        let column = self.unicode_columns[at]
            .as_ref()
            .map_or(byte - start, |columns| {
                columns.partition_point(|offset| *offset < byte - start)
            });
        Some(Location::new((at + 1) as u64, (column + 1) as u64))
    }
}
