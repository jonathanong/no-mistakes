use super::{check, check_with_suppressed};
use std::path::PathBuf;

fn source() -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/variants/final-assignment-fail/src/query.ts",
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
fn final_assignment_consumers_follow_fail_and_ignore_without_stale_offsets() {
    let source = source();
    for (case, opaque) in [
        ("final-assignment-fail", true),
        ("final-assignment-ignore", false),
    ] {
        let report = check("postgres-no-offset", case);
        let mut expected = vec![(line_of(&source, "id FROM unchanged_pages OFFSET"), "offset")];
        if opaque {
            expected.extend(
                [
                    "direct",
                    "computed",
                    "summarized",
                    "wrapped",
                    "nested",
                    "escaped",
                ]
                .map(|name| {
                    (
                        line_of(&source, &format!("query(sql`SELECT ${{{name}}}")),
                        "unanalyzable",
                    )
                }),
            );
        }
        expected.sort_unstable();
        let actual = report["rules"]
            .as_array()
            .unwrap()
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
fn final_assignment_consumers_keep_host_and_captured_branch_suppressions() {
    let source = source();
    let report = check_with_suppressed("postgres-no-offset", "final-assignment-fail", true);
    assert_eq!(report["rules"].as_array().unwrap().len(), 7, "{report}");
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
