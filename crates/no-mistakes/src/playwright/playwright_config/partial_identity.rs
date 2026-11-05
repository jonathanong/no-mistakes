use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(crate) fn config_name_with_sources(path: &Path, sources: &SourceStore) -> Option<String> {
    let source = SourceStore::read_prepared_or_open(Some(sources), path).ok()?;
    crate::playwright::ast::with_program(path, &source, |program, source| {
        let object = super::ast_nav::default_export_object(program)?;
        let value = super::ast_nav::property_expression(object, "name")?;
        super::literals::optional_string(value, source)
    })
    .ok()
    .flatten()
}
