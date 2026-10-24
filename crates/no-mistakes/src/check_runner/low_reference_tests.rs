use no_mistakes::codebase::rules::RuleFinding;
use std::path::PathBuf;
fn marked(source: &str, marker: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| line.contains(marker).then_some(i + 1))
        .collect()
}
fn lines(rows: &[RuleFinding]) -> Vec<usize> {
    rows.iter().map(|row| row.line).collect()
}
#[test]
fn literal_scalar_reads_stay_decidable_and_tagged_assignment_results_cannot_escape() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/low-reference-uses",
    );
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let missing = marked(&source, "EXPECT_MISSING_");
    let unknown = marked(&source, "EXPECT_UNKNOWN_");
    for policy in ["report", "advise", "ignore"] {
        let result = super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(format!(".no-mistakes-low-{policy}.yml"))),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        let mut expected = missing.clone();
        if policy == "report" {
            expected.extend(&unknown);
            expected.sort_unstable();
        }
        assert_eq!(
            lines(&result.rules),
            expected,
            "{policy}: {:?}",
            result.rules
        );
        assert_eq!(
            lines(&result.advisories),
            if policy == "advise" {
                unknown.clone()
            } else {
                Vec::new()
            },
            "{policy}"
        );
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }
}

#[test]
fn typed_executor_low_adopters_ignore_erased_uses_and_keep_statement_locations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/low-typed-executors");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let rules = [
        "postgres-no-offset",
        "postgres-require-query-annotation",
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-transformed-uuid-predicates",
    ];
    let annotation = |rows: &[RuleFinding]| {
        serde_json::to_value(
            rows.iter()
                .filter(|row| row.rule == "postgres-require-query-annotation")
                .collect::<Vec<_>>(),
        )
        .unwrap()
    };
    for mode in ["default", "advise", "ignore"] {
        let result = super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(format!(".no-mistakes-{mode}.yml"))),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        // Union syntax demand must not change the annotation receiver projection.
        let annotation_only = super::run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(format!(".no-mistakes-annotation-{mode}.yml"))),
            None,
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(
            annotation(&result.rules),
            annotation(&annotation_only.rules),
            "{mode}"
        );
        assert_eq!(
            annotation(&result.advisories),
            annotation(&annotation_only.advisories),
            "{mode}"
        );
        for rule in rules {
            let mut unknown = marked(&source, "// UNKNOWN");
            if matches!(
                rule,
                "postgres-no-offset"
                    | "postgres-require-query-annotation"
                    | "postgres-no-transformed-uuid-predicates"
            ) {
                unknown.extend(marked(&source, "// FALLBACK_UNKNOWN"));
            }
            // Annotation uses its own embedded projection: a visible head is
            // enough, and executor escapes are outside that rule's obligation.
            if rule != "postgres-require-query-annotation" {
                for marker in [
                    "// OUTSIDE",
                    "// ESCAPE",
                    "// RUNTIME_TYPE",
                    "// FACTORY",
                    "// UNSUPPORTED_TYPE",
                ] {
                    unknown.extend(marked(&source, marker));
                }
            }
            unknown.sort_unstable();
            let violations = if rule == "postgres-no-offset" {
                marked(&source, "// OFFSET")
            } else {
                Vec::new()
            };
            let mut expected = violations;
            if mode == "default" {
                expected.extend(&unknown);
            }
            expected.sort_unstable();
            let actual = lines(
                &result
                    .rules
                    .iter()
                    .filter(|row| row.rule == rule)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            assert_eq!(actual, expected, "{mode} {rule}: {:?}", result.rules);
            let advice = lines(
                &result
                    .advisories
                    .iter()
                    .filter(|row| row.rule == rule)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            assert_eq!(
                advice,
                if mode == "advise" {
                    unknown
                } else {
                    Vec::new()
                },
                "{mode} {rule}"
            );
        }
    }
}
