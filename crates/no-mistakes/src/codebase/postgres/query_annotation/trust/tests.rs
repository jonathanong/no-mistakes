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
fn casing_variants_are_not_trusted_without_a_configured_import() {
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
    assert!(
        tags.is_empty(),
        "a sql-shaped identifier with no configured import is unknown: {tags:?}"
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

#[test]
fn named_default_and_type_only_imports_preserve_existing_tag_trust() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-import-tags.mts");
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
        BTreeSet::from(["SQL", "anyTag", "sql"].map(str::to_string))
    );
}

#[test]
fn commonjs_require_of_sql_template_strings_is_trusted_under_its_local_name() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-require-tag.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let tags = crate::ast::with_program(&path, &source, |program, _| {
        collect(
            program,
            &super::super::super::EmbeddedSqlOptions::configured("", &[]),
        )
    })
    .unwrap();
    assert_eq!(tags, BTreeSet::from(["tag".to_string()]));
}

#[test]
fn canonical_module_tags_preserve_legacy_fallback_without_widening_shadows() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-legacy-local-tags.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let tags =
        crate::ast::with_program(&path, &source, |program, _| legacy_local_tags(program)).unwrap();
    assert_eq!(
        tags.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["String"]
    );
    for offset in tags.values() {
        assert!(source[*offset as usize..].starts_with("function"));
    }
    let class_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-default-class-tag.mts");
    let class_source = std::fs::read_to_string(&class_path).unwrap();
    let class_tags = crate::ast::with_program(&class_path, &class_source, |program, _| {
        legacy_local_tags(program)
    })
    .unwrap();
    assert!(
        class_tags.is_empty(),
        "a default class is not the intrinsic String tag: {class_tags:?}"
    );
}
