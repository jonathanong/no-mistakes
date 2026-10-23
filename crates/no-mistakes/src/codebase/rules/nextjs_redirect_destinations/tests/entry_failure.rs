use super::effort::run_prepared;
use super::{config, fixture, fixture_files, RuleFinding};
use crate::codebase::rules::analysis_policy::prepare_rule_analysis;
use crate::codebase::rules::source_store_for_files;
use crate::config::v2::NoMistakesConfig;

fn assert_policy(
    findings: &[RuleFinding],
    advisories: &[RuleFinding],
    policy: &str,
    expected_error: &str,
    detail: &str,
    line: usize,
) {
    let outcome = match policy {
        "report" => {
            assert!(advisories.is_empty(), "{advisories:?}");
            findings
        }
        "advise" => {
            assert!(findings.is_empty(), "{findings:?}");
            advisories
        }
        "ignore" => {
            assert!(findings.is_empty(), "{findings:?}");
            assert!(advisories.is_empty(), "{advisories:?}");
            return;
        }
        _ => unreachable!(),
    };
    assert_eq!(outcome.len(), 1, "{outcome:?}");
    assert_eq!(outcome[0].file, "next.config.ts");
    assert_eq!(outcome[0].line, line);
    assert!(outcome[0].message.contains("next.config.ts"), "{outcome:?}");
    assert!(outcome[0].message.contains(expected_error), "{outcome:?}");
    assert!(outcome[0].message.contains(detail), "{outcome:?}");
    assert!(outcome[0].message.contains("fix the file"), "{outcome:?}");
}

#[test]
fn malformed_and_invalid_utf8_entry_configs_follow_each_policy_at_both_efforts() {
    for effort in ["low", "max"] {
        for policy in ["report", "advise", "ignore"] {
            let options = format!("effort: {effort}\nunanalyzable: {policy}");
            let (findings, advisories) = run_prepared("entry-malformed", &options);
            assert_policy(
                &findings,
                &advisories,
                policy,
                "could not parse config",
                "Unexpected token",
                2,
            );
            let (findings, advisories) = run_prepared("entry-invalid-utf8", &options);
            assert_policy(
                &findings,
                &advisories,
                policy,
                "could not read config",
                "invalid utf-8 sequence",
                1,
            );
        }
    }
}

#[test]
fn parse_failure_anchor_skips_leading_comments_and_falls_back_for_comment_only_source() {
    for (name, detail, line) in [
        ("entry-malformed-comment-prefix", "Unexpected token", 5),
        ("entry-malformed-unterminated-comment", "Unterminated", 1),
    ] {
        let (findings, advisories) = run_prepared(name, "{ effort: max }");
        assert_policy(
            &findings,
            &advisories,
            "report",
            "could not parse config",
            detail,
            line,
        );
    }
}

#[test]
fn missing_entry_config_read_follows_each_policy_at_both_efforts() {
    let root = fixture("entry-missing-read");
    // Inventory declares a config path that cannot be read; no fixture is generated at runtime.
    let files = vec![root.join("next.config.ts")];
    let sources = source_store_for_files(&files);
    let expected_error = sources.read_path(&files[0]).unwrap_err().to_string();
    for effort in ["low", "max"] {
        for policy in ["report", "advise", "ignore"] {
            let mut settings: NoMistakesConfig =
                config(&format!("{{ effort: {effort}, unanalyzable: {policy} }}"));
            prepare_rule_analysis(&root, &mut settings, &files, &sources).unwrap();
            let findings =
                super::super::check_with_files_and_sources(&root, &settings, &files, &sources)
                    .unwrap();
            let advisories =
                super::super::advisories_with_files_and_sources(&root, &settings, &files, &sources)
                    .unwrap();
            assert_policy(
                &findings,
                &advisories,
                policy,
                "could not read config",
                &expected_error,
                1,
            );
        }
    }
}

#[test]
fn entry_parse_failure_honors_file_line_and_next_line_directives() {
    for effort in ["low", "max"] {
        for policy in ["report", "advise"] {
            let options = format!("effort: {effort}\nunanalyzable: {policy}");
            for name in [
                "entry-malformed-file",
                "entry-malformed-line",
                "entry-malformed-next-line",
            ] {
                let root = fixture(name);
                let files = fixture_files(&root);
                let sources = source_store_for_files(&files);
                let mut settings = config(&options);
                prepare_rule_analysis(&root, &mut settings, &files, &sources).unwrap();
                // This public diagnostic boundary applies suppression to both channels.
                let diagnostics = super::super::diagnostics_with_files_and_sources(
                    &root, &settings, &files, &sources, None, false,
                )
                .unwrap();
                assert!(
                    diagnostics.findings.is_empty(),
                    "{name}: {:?}",
                    diagnostics.findings
                );
                assert!(
                    diagnostics.advisories.is_empty(),
                    "{name}: {:?}",
                    diagnostics.advisories
                );
            }
        }
    }
}
