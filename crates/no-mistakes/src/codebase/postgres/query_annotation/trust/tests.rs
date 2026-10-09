use super::*;
use std::path::PathBuf;

#[test]
fn imported_tag_trust_and_module_writes_respect_lexical_bindings() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-trust.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = super::super::super::EmbeddedSqlOptions::configured("@consumer/db", &[])
        .with_trusted_sql_tags(&[super::super::super::TrustedSqlTag {
            module: "@consumer/db".into(),
            name: "sql".into(),
        }]);
    let (trusted, written) = crate::ast::with_program(&path, &source, |program, _| {
        (collect(program, &options), reassigned(program))
    })
    .unwrap();
    assert_eq!(
        trusted,
        BTreeSet::from(["sql".into(), "customQuery".into(), "subQuery".into()])
    );
    assert_eq!(
        written,
        HashSet::from(
            [
                "outerWrite",
                "arrowWrite",
                "loopWrite",
                "updateWrite",
                "destructureWrite",
                "restWrite",
                "defaultWrite",
                "defaultParamWrite",
                "arrowParamWrite",
                "helper",
                "exportedHelper",
                "globalWrite",
            ]
            .map(str::to_string)
        )
    );
}

#[test]
fn non_import_module_bindings_shadow_the_conventional_tag() {
    for fixture in [
        "query-annotation-shadowed-tag.mts",
        "query-annotation-import-equals-tag.mts",
        "query-annotation-default-function-tag.mts",
        "query-annotation-default-class-tag.mts",
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/embedded")
            .join(fixture);
        let source = std::fs::read_to_string(&path).unwrap();
        let tags = crate::ast::with_program(&path, &source, |program, _| {
            collect(
                program,
                &super::super::super::EmbeddedSqlOptions::configured("", &[]),
            )
        })
        .unwrap();
        assert!(tags.is_empty(), "{fixture}");
    }
}

#[test]
fn conventional_tags_preserve_casing_without_trusting_unrelated_imports() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-uppercase-tag.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let tags = crate::ast::with_program(&path, &source, |program, _| {
        collect(
            program,
            &super::super::super::EmbeddedSqlOptions::configured("", &[]),
        )
    })
    .unwrap();
    assert_eq!(
        tags,
        BTreeSet::from(["sql".into(), "SQL".into(), "sQl".into()])
    );
}

#[test]
fn named_default_function_redeclaration_is_a_module_write() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-default-function-write.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let names = crate::ast::with_program(&path, &source, |program, _| reassigned(program)).unwrap();
    assert_eq!(names, HashSet::from(["sql".to_string()]));
}
