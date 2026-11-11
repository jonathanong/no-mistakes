use super::*;
use crate::codebase::check_facts::tests::collect_file_facts;

fn saved_fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/fact-panic-isolation/fixture"),
    )
}

#[test]
fn fact_panic_helper_overrides_successful_ordinary_facts() {
    let fixture = saved_fixture();
    let root = fixture.path();
    let path = root.join("healthy.ts");
    let plan = CheckFactPlan {
        imports: true,
        ..Default::default()
    };
    let successful = collect_file_facts(root, &path, &plan, None).unwrap();
    assert!(successful.parsed);
    assert!(!successful.ts.imports.is_empty());
    let panicked = crate::codebase::fact_panic::tests::with_injected_panic(&path, || {
        collect_file_facts(root, &path, &plan, None).unwrap()
    });
    assert!(panicked.fact_extraction_panicked);
    let mut facts = FileIdMap::from_iter([(path.clone(), successful)]);
    let helpers = FileIdMap::from_iter([(path.clone(), panicked)]);
    extend_missing_helper_facts(&mut facts, helpers);
    let failed = &facts[&path];
    assert!(failed.fact_extraction_panicked);
    assert!(!failed.parsed);
    assert!(failed.ts.imports.is_empty());
    assert!(failed
        .ts
        .operational_error
        .as_ref()
        .unwrap()
        .contains("injected fact extraction failure"));
}

#[test]
fn ordinary_helper_failure_preserves_successful_ordinary_facts() {
    let fixture = saved_fixture();
    let root = fixture.path();
    let path = root.join("healthy.ts");
    let plan = CheckFactPlan {
        imports: true,
        ..Default::default()
    };
    let successful = collect_file_facts(root, &path, &plan, None).unwrap();
    let failed = CheckFileFacts {
        parse_error: Some("strict runner parse failed".into()),
        ..Default::default()
    };
    let mut facts = FileIdMap::from_iter([(path.clone(), successful)]);
    extend_missing_helper_facts(&mut facts, FileIdMap::from_iter([(path.clone(), failed)]));
    let kept = &facts[&path];
    assert!(kept.parsed);
    assert!(!kept.ts.imports.is_empty());
    assert!(kept.parse_error.is_none());
    assert!(!kept.fact_extraction_panicked);
}
