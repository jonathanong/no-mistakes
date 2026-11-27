use crate::codebase::check_facts::{
    sql_source_prerequisites::{SqlProjection, SqlSourceKey},
    CheckFileFacts,
};
use crate::codebase::ts_source::FileIdMap;
use std::path::Path;

pub(crate) fn source_demanded(
    files: &FileIdMap<CheckFileFacts>,
    path: &Path,
    key: &SqlSourceKey,
) -> bool {
    if !matches!(
        key.projection,
        SqlProjection::LowSyntax | SqlProjection::AnnotationMax
    ) {
        return true;
    }
    let Some(file) = files.get(path) else {
        return true;
    };
    if let Some(declared) = &file.sql_source_prerequisites.declared {
        return declared.contains(key);
    }
    match key.projection {
        SqlProjection::LowSyntax
            if !file
                .postgres()
                .sql_identifier_syntax
                .low_applications
                .is_empty() =>
        {
            key.application.as_ref().is_some_and(|(rule, index)| {
                file.postgres()
                    .sql_identifier_syntax
                    .low_applications
                    .iter()
                    .any(|(profile, owner, ordinal)| {
                        profile == &key.profile && owner == rule && ordinal == index
                    })
            })
        }
        SqlProjection::AnnotationMax if !file.postgres().query_annotation.is_empty() => file
            .postgres()
            .query_annotation
            .iter()
            .any(|(profile, _)| profile == &key.profile),
        _ => true,
    }
}
