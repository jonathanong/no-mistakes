//! An invalid SQL encoding is file-local uncertainty, never a failed invocation.
use crate::codebase::postgres::{
    SqlSchemaFileFacts, SqlStatementFileFacts, SqlUnanalyzableStatement,
};
use std::path::Path;

pub(in crate::codebase::postgres) fn failure(path: &Path) -> SqlSchemaFileFacts {
    SqlSchemaFileFacts {
        path: path.to_path_buf(),
        unanalyzable_statements: vec![marker()],
        ..Default::default()
    }
}

pub(in crate::codebase::postgres) fn statement_failure(path: &Path) -> SqlStatementFileFacts {
    SqlStatementFileFacts {
        path: path.to_path_buf(),
        parse_failed: true,
        parse_failures: vec![marker()],
        ..Default::default()
    }
}

fn marker() -> SqlUnanalyzableStatement {
    SqlUnanalyzableStatement {
        line: 1,
        column: 1,
        head: "invalid UTF-8 SQL file; save the file as UTF-8".into(),
        valid_unsupported: false,
        procedural_body: false,
        source_order: vec![0],
        insert_keyword_count: 0,
        recovered_insert_count: 0,
        has_top_level_not_exists: false,
        schema_unanalyzable: true,
        statement_unanalyzable: true,
    }
}

#[cfg(test)]
mod tests;
