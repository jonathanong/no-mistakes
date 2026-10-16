use super::*;

#[test]
fn custom_type_imports_are_hoisted_before_annotation_collection() {
    let root = root().with_file_name("typed-import-order-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    for (file, line) in [
        ("early.mts", 5),
        ("late.mts", 4),
        ("mixed.mts", 4),
        ("late-shapes.mts", 4),
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.file == file && finding.line == line),
            "missing {file}:{line}: {findings:#?}"
        );
        let report =
            crate::effects_query::run(&root, None, None, "postgres", Path::new(file), &[], None)
                .unwrap();
        assert_eq!(
            report
                .call_sites
                .iter()
                .map(|site| (site.file.as_str(), site.line))
                .collect::<Vec<_>>(),
            [(file, line)],
            "{report:#?}"
        );
    }
    // A custom import cannot lend its identity to another file's builtin or type parameter.
    assert!(
        !findings
            .iter()
            .any(|finding| finding.file == "controls.mts"),
        "{findings:#?}"
    );
}

#[test]
fn hoisted_type_imports_do_not_create_runtime_import_identities() {
    use crate::codebase::dependencies::extract::{
        extract_import_facts_from_program, CallTargetIdentity,
    };
    let root = root().with_file_name("typed-import-order-fixture");
    let source = std::fs::read_to_string(root.join("runtime-identity.mts")).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program(&parsed.program);
    let calls = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "NonNullable")
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].target_identity, CallTargetIdentity::Global);
}
