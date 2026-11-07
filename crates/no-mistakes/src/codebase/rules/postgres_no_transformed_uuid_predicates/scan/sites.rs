use super::super::groups::Site;
use crate::codebase::postgres::statements::{SqlFactSite, SqlTransformedColumnUseFact};
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn project(
    root: &Path,
    file: &SqlStatementFileFacts,
    index: usize,
    execution: usize,
    use_: &SqlTransformedColumnUseFact,
) -> Site {
    let position = file
        .variant_locations
        .as_ref()
        .and_then(|locations| locations.position(SqlFactSite::TransformedExpression(index)));
    let path = position
        .and_then(|position| position.source_path.as_deref())
        .unwrap_or(&file.path);
    (
        relative_slash_path(root, path),
        position.and_then(|position| position.source_offset),
        use_.expression_site,
        file.variant_locations
            .as_ref()
            .map(|locations| (true, locations.call_index))
            .or_else(|| {
                (file
                    .path
                    .extension()
                    .is_none_or(|extension| extension != "sql"))
                .then_some((false, execution))
            }),
    )
}
