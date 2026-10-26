use super::*;

#[test]
fn ignore_cannot_hide_consumer_document_or_reference_failures() {
    let files: Vec<_> = [
        "schemas.json",
        "malformed.json",
        "duplicate.json",
        "numeric.json",
        "removed.json",
    ]
    .map(|file| root().join(file))
    .into();
    let sources = super::super::super::source_store_for_files(&files);
    let invalid = [
        ("malformed.json", "", "invalid JSON"),
        ("duplicate.json", "", "duplicate object key"),
        ("numeric.json", "", "loses decimal precision"),
        ("removed.json", "", "cannot read schema"),
        ("outside.json", "", "outside the configured rule scope"),
        ("../schemas.json", "", "root-relative"),
        ("schemas.json", "/absent", "does not select a schema"),
        ("schemas.json", "missing-slash", "RFC 6901"),
        ("schemas.json", "/bad~2escape", "pointer escape"),
    ];
    for (file, pointer, expected) in invalid {
        // A nested producer would otherwise take Ignore's zero-report branch.
        let mut pair = contract("nested", "nested");
        pair.consumer = DeclaredPayloadSchema {
            file: file.into(),
            pointer: pointer.into(),
        };
        let config = NoMistakesConfig {
            rules: vec![rule(Effort::Low, Unanalyzable::Ignore, vec![pair])],
            ..NoMistakesConfig::default()
        };
        let output =
            diagnostics_with_files_and_sources(&root(), &config, &files, &sources, true).unwrap();
        assert_eq!(output.findings.len(), 1, "{file}: {:?}", output.findings);
        assert!(output.advisories.is_empty());
        assert!(
            output.findings[0].message.contains(expected),
            "{}",
            output.findings[0].message
        );
        assert!(output.findings[0].message.contains("unproven consumer"));
    }
}

#[test]
fn max_keeps_its_original_producer_schema_error_precedence() {
    let files = [root().join("schemas.json"), root().join("malformed.json")];
    let sources = super::super::super::source_store_for_files(&files);
    let pair = || {
        let mut pair = contract("invalid", "nested");
        pair.consumer.file = "malformed.json".into();
        pair.consumer.pointer.clear();
        pair
    };
    for resolved in [None, Some(EffectiveRuleAnalysis::default())] {
        let mut application = rule(Effort::Max, Unanalyzable::Report, vec![pair()]);
        application.resolved_analysis = resolved;
        let config = NoMistakesConfig {
            rules: vec![application],
            ..NoMistakesConfig::default()
        };
        let output =
            diagnostics_with_files_and_sources(&root(), &config, &files, &sources, true).unwrap();
        assert_eq!(output.findings.len(), 1);
        assert!(output.findings[0]
            .message
            .contains("unproven producer: $: unsupported keyword `format`"));
        assert_eq!(
            output.findings,
            check_with_files_and_sources(&root(), &config, &files, &sources).unwrap()
        );
    }
}

#[test]
fn suppression_remains_at_the_requested_boundary_for_document_errors() {
    let files = ["file.json", "line.json", "next.json"].map(|file| root().join(file));
    let sources = super::super::super::source_store_for_files(&files);
    let pairs = || {
        files
            .iter()
            .map(|file| {
                let mut pair = contract("scalar", "scalar");
                pair.producer.file = file.file_name().unwrap().to_str().unwrap().into();
                pair.producer.pointer.clear();
                pair
            })
            .collect()
    };
    for policy in [Unanalyzable::Report, Unanalyzable::Advise] {
        let config = NoMistakesConfig {
            rules: vec![rule(Effort::Low, policy, pairs())],
            ..NoMistakesConfig::default()
        };
        let deferred =
            diagnostics_with_files_and_sources(&root(), &config, &files, &sources, true).unwrap();
        assert_eq!(deferred.findings.len(), 3);
        let suppressed =
            diagnostics_with_files_and_sources(&root(), &config, &files, &sources, false).unwrap();
        assert!(suppressed.findings.is_empty());
        assert!(suppressed.advisories.is_empty());
    }
}
