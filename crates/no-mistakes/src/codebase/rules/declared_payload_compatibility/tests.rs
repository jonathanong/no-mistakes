use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use serde_json::Value;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/declared-payload-compatibility")
            .join(name),
    )
}

fn data(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixture(name)).unwrap()).unwrap()
}

#[test]
fn directional_schema_inclusion_matches_saved_cases() {
    for case in data("compatibility.json").as_array().unwrap() {
        let producer = schema::parse(&case["producer"], "$", 0).unwrap();
        let consumer = schema::parse(&case["consumer"], "$", 0).unwrap();
        let result = compare::subset(&producer, &consumer, "$");
        let expected = case["expected"].as_str().unwrap();
        if expected.is_empty() {
            assert!(result.is_ok(), "{}: {result:?}", case["name"]);
        } else {
            assert!(result.unwrap_err().contains(expected), "{}", case["name"]);
        }
    }
}

#[test]
fn unsupported_and_malformed_schema_forms_never_claim_compatibility() {
    for case in data("unsupported.json").as_array().unwrap() {
        let error = schema::parse(&case["schema"], "$", 0).unwrap_err();
        assert!(
            error.contains(case["expected"].as_str().unwrap()),
            "{}: {error}",
            case["name"]
        );
    }
}

#[test]
fn configured_http_and_queue_pairs_share_one_physical_document_read() {
    for (name, expected) in [("pass", 0), ("fail", 1)] {
        let root = fixture(name);
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
        let files = vec![root.join("schemas.json")];
        let sources = super::super::source_store_for_files(&files);
        let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
        assert_eq!(findings.len(), expected, "{findings:?}");
        assert_eq!(sources.physical_read_count(), 1);
        if let Some(finding) = findings.first() {
            assert_eq!(finding.file, "schemas.json");
            assert_eq!(finding.target.as_deref(), Some("queue user-notifications"));
            assert!(finding.message.contains("userId"));
        }
        assert_eq!(check_with_files(&root, &config, &files).unwrap(), findings);
    }
}

#[test]
fn invalid_references_emit_unproven_findings() {
    let root = fixture("");
    // The authoritative inventory may retain a deleted file. Its failure must
    // be shared by SourceStore, not recovered through an independent read.
    let files = vec![
        root.join("pass/schemas.json"),
        root.join("malformed.json"),
        root.join("unreadable.json"),
    ];
    let sources = super::super::source_store_for_files(&files);
    let documents = document::Documents::default();
    let allowed = files
        .iter()
        .map(|p| relative_slash_path(&root, p))
        .collect();
    for case in data("references.json").as_array().unwrap() {
        let valid = || DeclaredPayloadSchema {
            file: "pass/schemas.json".into(),
            pointer: "/request".into(),
        };
        let invalid = DeclaredPayloadSchema {
            file: case["file"].as_str().unwrap().into(),
            pointer: case["pointer"].as_str().unwrap().into(),
        };
        let mut contract = DeclaredPayloadContract {
            name: case["name"].as_str().unwrap().into(),
            producer: valid(),
            consumer: valid(),
        };
        if case["consumer"] == true {
            contract.consumer = invalid;
        } else {
            contract.producer = invalid;
        }
        if contract.name == "empty-name" {
            contract.name.clear();
        }
        let (_, error) =
            check_contract(&root, &contract, &allowed, &sources, &documents).unwrap_err();
        assert!(
            error.contains(case["reason"].as_str().unwrap()),
            "{}: {error}",
            case["name"]
        );
        assert!(error.contains("unproven"));
    }
    assert_eq!(sources.physical_read_count(), 3);
    assert_eq!(
        documents.parses.load(std::sync::atomic::Ordering::Relaxed),
        2
    );
}

#[test]
fn raw_json_precision_and_duplicate_keys_are_validated_before_comparison() {
    for case in data("documents.json").as_array().unwrap() {
        let result = document::parse(case["source"].as_str().unwrap());
        let expected = case["expected"].as_str().unwrap();
        if expected.is_empty() {
            assert!(result.is_ok(), "{}: {result:?}", case["name"]);
        } else {
            assert!(result.unwrap_err().contains(expected), "{}", case["name"]);
        }
    }
}

#[test]
fn many_parallel_pairs_reuse_one_json_parse() {
    let root = fixture("pass");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let options: DeclaredPayloadCompatibilityOptions = config.rules[0].try_rule_options().unwrap();
    let files = vec![root.join("schemas.json")];
    let sources = super::super::source_store_for_files(&files);
    let documents = document::Documents::default();
    let allowed = files
        .iter()
        .map(|p| relative_slash_path(&root, p))
        .collect();
    (0..128).into_par_iter().for_each(|_| {
        for contract in &options.contracts {
            check_contract(&root, contract, &allowed, &sources, &documents).unwrap();
        }
    });
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(
        documents.parses.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn custom_messages_keep_compatibility_classification() {
    let root = fixture("fail");
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules[0].message = Some("Migrate both services together".into());
    let findings = check_with_files(&root, &config, &[root.join("schemas.json")]).unwrap();
    assert!(findings[0]
        .message
        .starts_with("Migrate both services together: contract"));
    assert!(findings[0].message.contains("incompatible"));
}

#[test]
fn file_line_and_next_line_suppression_ignore_directive_looking_string_data() {
    let root = fixture("suppression");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = ["file.json", "line.json", "next.json", "string.json"].map(|name| root.join(name));
    let findings =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "string.json");
}

#[test]
fn strict_options_reject_wrong_shapes_and_unknown_fields() {
    let options: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&std::fs::read_to_string(fixture("invalid-options.yml")).unwrap())
            .unwrap();
    for options in options {
        let config = NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.into(),
                scope: Some(RuleScope::Repository),
                options,
                ..Default::default()
            }],
            ..Default::default()
        };
        let error = check_with_files(&fixture(""), &config, &[]).unwrap_err();
        assert!(error.to_string().contains("invalid options"), "{error:#}");
    }
}

#[test]
fn excluded_and_out_of_scope_declarations_fail_closed() {
    let root = fixture("fail");
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules[0].exclude = vec!["schemas.json".into()];
    let findings = check_with_files(&root, &config, &[root.join("schemas.json")]).unwrap();
    assert_eq!(findings.len(), 4);
    assert!(findings.iter().all(|f| f.message.contains("unproven")));
}

#[test]
fn dispatches_through_prepared_filesystem_rule_registry() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("schemas.json")];
    let findings =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config, &files).unwrap();
    assert_eq!(findings, check_with_files(&root, &config, &files).unwrap());
    assert_eq!(findings.len(), 1);
}

#[test]
fn defensive_numeric_validation_rejects_malformed_saved_tokens() {
    for case in data("invalid-tokens.json").as_array().unwrap() {
        let error = numeric::validate(case["source"].as_str().unwrap()).unwrap_err();
        assert!(
            error.contains(case["expected"].as_str().unwrap()),
            "{}: {error}",
            case["name"]
        );
    }
}
