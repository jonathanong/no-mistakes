use oxc_allocator::Allocator;
use oxc_span::{SourceType, Span};
use std::path::{Path, PathBuf};

#[test]
fn source_sensitive_module_names_keep_legacy_fatal_diagnostic_and_empty_program() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    for name in [
        "export.ts",
        "import.ts",
        "reexport-local.ts",
        "reexport-name.ts",
        "export-all.ts",
        "namespace.ts",
        "external-module.ts",
        "global.ts",
        "exported-namespace.ts",
        "prefix.ts",
        "comments.ts",
        "diagnostics.ts",
        "whitespace.ts",
        "fatal-records/complete-all.ts",
        "fatal-records/complete-export.ts",
        "fatal-records/complete-import.ts",
        "fatal-records/complete-reexport.ts",
        "fatal-records/malformed-alias.ts",
        "fatal-records/missing-brace.ts",
        "fatal-records/missing-import-alias.ts",
        "fatal-records/missing-reexport-source.ts",
        "fatal-records/earliest.ts",
        "fatal-prefix/nested-later-fatal.ts",
        "fatal-prefix/nested-missing-brace.ts",
        "fatal-prefix/exported-nested-later.ts",
        "fatal-prefix/declared-dotted.ts",
        "fatal-prefix/module.ts",
        "fatal-prefix/declared-export.ts",
        "fatal-prefix/nested-import.ts",
        "fatal-prefix/nested-reexport.ts",
        "fatal-prefix/nested-all.ts",
        "fatal-prefix/prefix-clauses.ts",
        "fatal-prefix/comments.ts",
        "fatal-prefix/contextual-type.ts",
        "fatal-prefix/contextual-let.ts",
        "fatal-prefix/contextual-await.ts",
        "fatal-prefix/contextual-yield.ts",
        "fatal-prefix/contextual-interface.ts",
        "fatal-prefix/contextual-implements.ts",
        "fatal-prefix/contextual-package.ts",
        "fatal-prefix/contextual-private.ts",
        "fatal-prefix/contextual-protected.ts",
        "fatal-prefix/contextual-public.ts",
        "fatal-prefix/contextual-static.ts",
        "fatal-prefix/contextual-dotted.ts",
    ] {
        let source = std::fs::read_to_string(fixture.path().join(name)).unwrap();
        let allocator = Allocator::default();
        let parsed = crate::ast::parse(Path::new(name), &allocator, &source, SourceType::ts());
        assert!(parsed.fatal_error, "{name}");
        assert_eq!(
            parsed.diagnostics.len(),
            if name == "diagnostics.ts" { 2 } else { 1 },
            "{name}"
        );
        let diagnostic = parsed.diagnostics.last().unwrap();
        assert_eq!(
            diagnostic.message,
            "An export name cannot include a unicode lone surrogate"
        );
        let literal = "\"\\u{D800}\\u{DC00}\"";
        // The comments control contains an earlier fake name that must be skipped.
        let start = if name == "fatal-prefix/comments.ts" {
            source.rfind(literal)
        } else {
            source.find(literal)
        }
        .unwrap() as u32;
        assert_eq!(diagnostic.labels.len(), 1);
        assert_eq!(diagnostic.labels[0].offset(), start);
        assert_eq!(diagnostic.labels[0].len(), literal.len() as u32);
        assert_eq!(parsed.program.span, Span::default());
        assert_eq!(parsed.program.source_text, source);
        assert!(parsed.program.body.is_empty());
        assert!(parsed.program.directives.is_empty());
        assert!(parsed.program.hashbang.is_none());
        assert!(parsed.tokens.is_empty());
        if name == "comments.ts" {
            assert_eq!(parsed.program.comments.len(), 2);
        }
    }
}

#[test]
fn valid_module_names_and_namespace_literal_values_remain_available() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let source = std::fs::read_to_string(fixture.path().join("valid.ts")).unwrap();
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(Path::new("valid.ts"), &allocator, &source, SourceType::ts());
    assert!(!parsed.fatal_error);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.program.body.len(), 3);
}

#[test]
fn fatal_record_fallback_preserves_ordinary_module_names() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let source = std::fs::read_to_string(fixture.path().join("fatal-records/ordinary.ts")).unwrap();
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        Path::new("ordinary.ts"),
        &allocator,
        &source,
        SourceType::ts(),
    );
    assert!(parsed.fatal_error);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].message, "Unexpected token");
    assert_eq!(
        parsed.diagnostics[0].labels[0].offset(),
        source.find("=;").unwrap() as u32
    );
}

#[test]
fn namespace_prefix_proof_stops_at_unknown_syntax_and_skips_fake_names() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let rows: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.path().join("prefix-controls.json")).unwrap(),
    )
    .unwrap();
    for row in rows.as_array().unwrap() {
        let source = row["source"].as_str().unwrap();
        let expected = row["span"].as_array().map(|span| {
            Span::new(
                span[0].as_u64().unwrap() as u32,
                span[1].as_u64().unwrap() as u32,
            )
        });
        assert_eq!(
            super::prefix_names::first_legacy_name(source, row["limit"].as_u64().unwrap() as usize),
            expected,
            "{source:?}"
        );
    }
}

#[test]
fn non_typescript_fatal_sources_do_not_use_namespace_prefix_proof() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ast/js-string-identity/module-names");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let path = fixture.path().join("fatal-prefix/ordinary.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        &path,
        &allocator,
        &source,
        SourceType::from_path(&path).unwrap(),
    );
    assert!(parsed.fatal_error);
    assert_eq!(parsed.diagnostics[0].message, "Unexpected token");
}
