use super::*;
use crate::codebase::postgres::EmbeddedSqlSourcePosition;

pub(super) struct SqlText<'a> {
    pub(super) sql: &'a str,
    pub(super) positions: &'a [EmbeddedSqlSourcePosition],
    pub(super) base: u32,
}

pub(super) fn collect(
    comments: &mut Vec<PreparedComment>,
    scope: &AuditScope<'_>,
    path: &Path,
    text: SqlText<'_>,
) {
    let AuditScope {
        root,
        rule,
        selected,
        sources,
    } = *scope;
    let sql = text.sql;
    if !sql.contains("no-mistakes-disable-") {
        return;
    }
    for named in disable_directives(sql, true) {
        if named.rule != rule {
            continue;
        }
        let sql_line = match named.directive {
            DisableDirective::NextLine { line } => line,
            _ => continue,
        };
        let line_text = sql.lines().nth(sql_line as usize - 1).unwrap_or_default();
        let column = line_text.len() - line_text.trim_start().len() + 1;
        let origin = text
            .positions
            .partition_point(|origin| {
                (origin.sql_line, origin.sql_column) <= (sql_line, column as u32)
            })
            .checked_sub(1)
            .map(|index| &text.positions[index]);
        let path = origin
            .and_then(|origin| origin.source_path.as_deref())
            .unwrap_or(path);
        if !selected.contains(path) {
            continue;
        }
        let line = origin.map_or(sql_line + text.base.saturating_sub(1), |origin| {
            origin.source_line + sql_line - origin.sql_line
        });
        let Some(source) =
            crate::codebase::rules::suppression::read_directive_source(sources, path)
        else {
            continue;
        };
        let directive = DisableDirective::NextLine { line };
        if !has_disable_comment(&source, line.saturating_add(1), rule) {
            continue;
        }
        comments.push(PreparedComment {
            file: relative_slash_path(root, path),
            rule: rule.to_owned(),
            directive: project(directive),
            status: DirectiveStatus::Enabled,
        });
    }
}
