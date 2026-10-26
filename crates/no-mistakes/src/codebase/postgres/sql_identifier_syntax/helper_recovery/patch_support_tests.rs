use crate::codebase::{
    check_facts::CheckFileFacts,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub(super) fn prepare(names: &[&str]) -> FileIdMap<CheckFileFacts> {
    names
        .iter()
        .map(|name| {
            let path = super::tests::fixture().join("src").join(name);
            let source = std::fs::read_to_string(&path).unwrap();
            let facts = crate::ast::with_program(&path, &source, |program, _| {
                let ts = collect_file_facts_from_program(
                    &path,
                    TsFactPlan {
                        module_bindings: true,
                        ..Default::default()
                    },
                    &TsFactContext::default(),
                    &source,
                    program,
                    None,
                    None,
                );
                let syntax =
                    super::super::collect(program, &ts, &source, &[super::tests::profile()]);
                CheckFileFacts {
                    ts: Arc::new(ts),
                    sql_identifier_syntax: syntax,
                    ..Default::default()
                }
            })
            .unwrap();
            (path, facts)
        })
        .collect()
}
pub(super) fn resolve(
    files: &FileIdMap<CheckFileFacts>,
    from: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    let target = from
        .parent()?
        .join(specifier.strip_prefix("./")?)
        .with_extension("mts");
    files.contains_key(&target).then_some(target)
}
pub(super) fn binding(files: &FileIdMap<CheckFileFacts>, file: &Path, name: &str) -> usize {
    files
        .get(file)
        .unwrap()
        .ts
        .module_bindings
        .as_deref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| binding.name == name && binding.scope_id == 0)
        .unwrap()
        .id
}
