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

/// Criterion adapter for per-file call-site membership after the sites are sorted.
#[doc(hidden)]
pub fn benchmark_probe_call_site_files(file_count: usize) -> usize {
    let mut sites = Vec::with_capacity(file_count);
    for index in 0..file_count {
        sites.push(ResolvedCallSite {
            file: std::path::PathBuf::from(format!("/{index}.ts")),
            caller: None,
            caller_id: None,
            line: 1,
            offset: 0,
            invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
            source_callee: "f".to_string(),
            target: ResolvedCallTarget::Unknown,
        });
    }
    sites.sort_by(|left, right| left.file.cmp(&right.file));
    let index = index_sorted_call_sites_by_file(&sites);
    (0..file_count)
        .filter(|file| index.contains_key(std::path::Path::new(&format!("/{file}.ts"))))
        .count()
}
