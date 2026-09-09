/// Criterion adapter. Returns a size so the compiler cannot drop the index.
#[doc(hidden)]
pub fn benchmark_construct_callable_file_index(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> usize {
    let index = CallableFileIndex::from_facts(facts);
    index.aliases.len()
        + index.callable_bindings.len()
        + index.class_bindings.len()
        + index.imported.len()
        + index.exported.len()
}
