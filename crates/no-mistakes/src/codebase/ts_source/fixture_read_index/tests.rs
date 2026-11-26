use super::*;
use globset::{Glob, GlobSetBuilder};
use oxc_semantic::SemanticBuilder;

fn index(root: &Path, literal: &str) -> FixtureReadIndex {
    let mut tests = GlobSetBuilder::new();
    tests.add(Glob::new("*.{ts,js,cjs}").unwrap());
    let scope = FixtureReadScope::new(tests.build().unwrap(), vec![literal.into()]).unwrap();
    FixtureReadIndex::new(root.to_owned(), vec![scope])
}

#[test]
fn traces_only_configured_sources_and_reuses_cached_reads() {
    let root = crate::test_support::fixture_root("fixture-reads");
    let path = root.join("reads.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let literal = r#"loadFixture("direct")"#;
    let index = index(&root, literal);
    let change = ["/unrelated".into()];

    assert!(index.affects(&path, &source, literal, &change, false));
    assert!(!index.affects(&path, &source, literal, &change, true));
    assert!(!index.affects(&path, &source, literal, &change, false));

    let outside = root.with_file_name("unscoped").join("reads.ts");
    assert!(index.affects(&outside, &source, literal, &change, true));
    assert!(!format!("{index:?}").contains(literal));
}

#[test]
fn graph_observations_reuse_valid_semantics_and_keep_invalid_programs() {
    let root = crate::test_support::fixture_root("fixture-reads");
    let cases = [
        ("reads.ts", r#"loadFixture("direct")"#, false, false),
        (
            "semantic-error.js",
            r#"loadFixture("duplicate")"#,
            true,
            true,
        ),
        ("with.cjs", r#"loadFixture("scoped")"#, false, true),
    ];
    for (file, literal, diagnostics, keep) in cases {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let index = index(&root, literal);
        crate::ast::with_program(&path, &source, |program, source| {
            let semantic = SemanticBuilder::new()
                .with_build_nodes(true)
                .with_check_syntax_error(true)
                .build(program);
            assert_eq!(!semantic.diagnostics.is_empty(), diagnostics, "{file}");
            index.record(&path, source, program, Some(&semantic));
        })
        .unwrap();
        let change = ["/unrelated".into()];
        assert_eq!(index.affects(&path, &source, literal, &change, false), keep);
        let reads = FixtureReads::collect(&path, &source, &BTreeSet::from([literal.into()]));
        assert_eq!(reads.affects(&source, literal, &change), keep);
    }
}
