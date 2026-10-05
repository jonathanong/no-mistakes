use sqlparser::ast::{Select, Spanned};

pub(super) fn select_line(sql: &str, select: &Select, tables: &[String]) -> usize {
    let start = select.span().start;
    let source_at_start = sql
        .lines()
        .nth(start.line.saturating_sub(1) as usize)
        .and_then(|line| {
            let column = start.column.saturating_sub(1) as usize;
            line.char_indices()
                .nth(column)
                .and_then(|(byte, _)| line.get(byte..))
        });
    let standalone_table = source_at_start.is_some_and(|source| {
        source
            .get(..5)
            .is_some_and(|word| word.eq_ignore_ascii_case("TABLE"))
            && source
                .as_bytes()
                .get(5)
                .is_none_or(|byte| !byte.is_ascii_alphanumeric())
    });
    if standalone_table {
        start.line as usize
    } else {
        let name = tables.first().map(String::as_str).unwrap_or("select");
        // Comments and quoted text can mention the table (`/* lockDocuments */`);
        // prefer the first code token, then fall back to a substring search.
        super::super::lines::first_word_line(sql, name.rsplit('.').next().unwrap_or(name))
            .unwrap_or_else(|| super::super::lines::line_containing(sql, &[name]))
    }
}
