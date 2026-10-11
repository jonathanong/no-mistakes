use super::{check, check_with_suppressed};
use std::path::PathBuf;

fn source() -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/variants/final-interpolation-fail/src/query.ts",
    ))
    .unwrap()
}

fn line_of(source: &str, needle: &str) -> usize {
    source
        .lines()
        .position(|line| line.contains(needle))
        .unwrap()
        + 1
}

#[test]
fn final_fragment_mutations_follow_the_dynamic_sql_policy() {
    let source = source();
    for (case, opaque) in [
        ("final-interpolation-fail", true),
        ("final-interpolation-ignore", false),
    ] {
        let report = check("postgres-no-offset", case);
        let findings = report["rules"].as_array().unwrap();
        let mut expected = vec![(line_of(&source, "? sql`id FROM pages"), "offset")];
        if opaque {
            expected.extend(["mutated", "escaped", "summarized"].map(|name| {
                (
                    line_of(&source, &format!("query(sql`SELECT ${{{name}}}")),
                    "unanalyzable",
                )
            }));
        }
        expected.sort_unstable();
        let actual = findings
            .iter()
            .map(|finding| {
                (
                    finding["line"].as_u64().unwrap() as usize,
                    finding["target"]
                        .as_str()
                        .unwrap()
                        .split('#')
                        .next()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{case}: {report}");
    }
}

#[test]
fn final_interpolation_fallback_and_static_branch_tokens_keep_line_suppression() {
    let source = source();
    let report = check_with_suppressed("postgres-no-offset", "final-interpolation-fail", true);
    assert_eq!(report["rules"].as_array().unwrap().len(), 4, "{report}");
    let mut actual = report["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["directive"]["kind"], "nextLine", "{report}");
            finding["line"].as_u64().unwrap() as usize
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            line_of(&source, "? sql`id FROM suppressed_pages"),
            line_of(&source, "query(sql`SELECT ${suppressedMutation}"),
        ],
        "{report}"
    );
}
