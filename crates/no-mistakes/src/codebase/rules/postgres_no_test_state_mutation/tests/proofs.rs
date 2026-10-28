use super::*;

#[test]
fn helper_outside_configured_roots_stays_unresolved_with_source_in_inventory() {
    let root = fixture().join("effort-scope");
    let files = [
        root.join("tests/scope.test.ts"),
        root.join("helpers/sql.ts"),
        root.join("outside/sql.ts"),
    ];
    let source = std::fs::read_to_string(&files[0]).unwrap();
    let sources = super::super::super::source_store_for_files(&files);
    let line = |marker: &str| {
        source
            .lines()
            .position(|text| text.contains(marker))
            .unwrap() as u32
            + 1
    };
    for config_name in ["low.yml", "mixed.yml"] {
        let mut config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
        crate::codebase::rules::analysis_policy::prepare_rule_analysis(
            &root,
            &mut config,
            &files,
            &sources,
        )
        .unwrap();
        let rules = if config_name == "mixed.yml" {
            &[RULE_ID, "postgres-no-offset"][..]
        } else {
            &[RULE_ID][..]
        };
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            Arc::clone(&sources),
            &config,
            rules,
        )
        .unwrap()
        .with_fresh_sql_verdicts();
        let findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        let outside = &facts.ts[&files[2]].sql_identifier_syntax;
        assert!(
            !outside
                .application_keys()
                .iter()
                .any(|key| key.rule == RULE_ID),
            "{config_name}: outside source gained test-state ownership"
        );
        if config_name == "mixed.yml" {
            let test_profile = &facts.ts[&files[0]]
                .sql_identifier_syntax
                .application_keys()
                .into_iter()
                .find(|key| key.rule == RULE_ID)
                .unwrap()
                .profile;
            let companion = outside
                .application_keys()
                .into_iter()
                .find(|key| key.rule == "postgres-no-offset")
                .unwrap();
            assert_eq!(
                &companion.profile, test_profile,
                "companion and test-state profiles diverged"
            );
        }
        assert_eq!(
            !outside.recovery.functions.is_empty(),
            config_name == "mixed.yml",
            "{config_name}: companion demand did not collect outside helper"
        );
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                row.file == "tests/scope.test.ts"
                    && crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                        == line("outside helper remains advised")
                    && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
                    && row.advisory
            }),
            "{config_name}: outside helper lost uncertainty"
        );
        assert!(!findings.iter().any(|row| row.file == "outside/sql.ts"));
        assert!(findings.iter().any(|row| {
            row.file == "helpers/sql.ts" && row.target.as_deref() == Some("DROP TABLE")
        }));
    }
}

fn run(name: &str) -> (Vec<RuleFinding>, crate::codebase::check_facts::CheckFactMap) {
    let root = fixture().join("effort-proofs");
    let files = [
        root.join("tests/proofs.test.ts"),
        root.join("helpers/sql.ts"),
    ];
    let sources = super::super::super::source_store_for_files(&files);
    let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap()
    .with_fresh_sql_verdicts();
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    (findings, facts)
}

#[test]
fn bounded_helpers_keep_definite_mutations_and_residual_uncertainty() {
    let root = fixture().join("effort-proofs");
    let path = root.join("tests/proofs.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let helper_source = std::fs::read_to_string(root.join("helpers/sql.ts")).unwrap();
    let line = |marker: &str| {
        source
            .lines()
            .position(|text| text.contains(marker))
            .unwrap() as u32
            + 1
    };
    let (low, facts) = run("low.yml");
    assert!(
        facts.postgres.embedded.is_empty(),
        "low ran max embedded extraction"
    );
    let (max, _) = run("max.yml");
    for target in ["DROP TABLE", "pg_sleep"] {
        assert!(
            low.iter().any(|row| row.target.as_deref() == Some(target)),
            "{target}: {low:?}"
        );
        assert!(
            max.iter().any(|row| row.target.as_deref() == Some(target)),
            "{target}: {max:?}"
        );
    }
    let throw_origin = helper_source
        .lines()
        .position(|text| text.contains("throw effect DROP origin"))
        .unwrap()
        + 1;
    assert!(
        low.iter().any(|row| {
            row.file == "helpers/sql.ts"
                && row.line == throw_origin
                && row.target.as_deref() == Some("DROP TABLE")
        }),
        "throw argument SQL finding lost: {low:?}"
    );
    for (marker, expected) in [
        (
            "recovered safe helper",
            crate::codebase::postgres::SqlVerdictStatus::ProvenClear,
        ),
        (
            "recovered DROP helper",
            crate::codebase::postgres::SqlVerdictStatus::Violation,
        ),
        (
            "recovered sleep helper",
            crate::codebase::postgres::SqlVerdictStatus::Violation,
        ),
    ] {
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    == line(marker)
                    && row.status == expected
                    && row.evidence.is_some()
                    && row.residual_reasons.is_empty()
            }),
            "{marker}: {:?}",
            facts.sql_verdicts()
        );
    }
    for marker in [
        "uncertain helper branch",
        "invalid helper SQL",
        "exported mutable SQL",
        "external wrapper",
        "bounded depth residual",
        "escaped helper",
        "throw argument has SQL effect",
    ] {
        assert!(
            facts.sql_verdicts().iter().any(|row| {
                crate::codebase::ts_source::byte_offset_to_line(&source, row.span.0 as usize)
                    == line(marker)
                    && row.advisory
                    && row.status == crate::codebase::postgres::SqlVerdictStatus::Abstain
            }),
            "{marker}: {:?}",
            facts.sql_verdicts()
        );
    }
}
