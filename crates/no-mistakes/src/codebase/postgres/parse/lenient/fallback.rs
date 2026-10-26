use super::{parse_with_sources, LocatedStatement};

pub(super) fn statements(
    sql: &str,
    allow_concurrent_detach: bool,
    procedural_body: bool,
) -> Vec<LocatedStatement> {
    let chunks = super::super::located_statements(sql);
    if chunks.len() <= 1 {
        return chunks
            .into_iter()
            .map(|(line, column, text)| LocatedStatement::failed_source(&text, line, column, 0))
            .collect();
    }
    chunks
        .into_iter()
        .enumerate()
        .scan(
            (0usize, 1usize, 1usize),
            |(offset, line, column), (ordinal, (_, _, text))| {
                let start = *offset + sql[*offset..].find(&text).unwrap();
                advance(&sql[*offset..start], line, column);
                let chunk = format!(
                    "{}{}{text}",
                    "\n".repeat(line.saturating_sub(1)),
                    " ".repeat(column.saturating_sub(1))
                );
                advance(&text, line, column);
                *offset = start + text.len();
                let mut parsed =
                    parse_with_sources(&chunk, true, allow_concurrent_detach, procedural_body);
                for failure in parsed
                    .iter_mut()
                    .filter_map(|item| item.recovery_failure.as_mut())
                {
                    // Re-parsing one lexical chunk starts at zero; retain its nested path.
                    failure.source_order[0] = ordinal;
                }
                Some(parsed)
            },
        )
        .flatten()
        .collect()
}

fn advance(text: &str, line: &mut usize, column: &mut usize) {
    for character in text.chars() {
        if character == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    }
}
