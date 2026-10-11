use super::{check, check_with_suppressed};
use std::path::PathBuf;

fn source() -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/variants/final-rebinding-fail/src/query.ts",
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
fn final_rebinding_checks_the_captured_offset_under_fail_and_ignore() {
    let source = source();
    for (case, opaque) in [
        ("final-rebinding-fail", true),
        ("final-rebinding-ignore", false),
    ] {
        let report = check("postgres-no-offset", case);
        let mut expected = [
            "id FROM replacements OFFSET 9",
            "id FROM pages OFFSET 1",
            "id FROM wrapped_pages OFFSET 2",
            "id FROM aliases OFFSET 3",
        ]
        .map(|needle| (line_of(&source, needle), "offset"))
        .to_vec();
        if opaque {
            expected.extend(["appended", "summarized", "escaped"].map(|name| {
                (
                    line_of(&source, &format!("query(sql`SELECT ${{{name}}}")),
                    "unanalyzable",
                )
            }));
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
fn final_rebinding_preserves_branch_host_and_dynamic_rhs_suppressions() {
    let source = source();
    let report = check_with_suppressed("postgres-no-offset", "final-rebinding-fail", true);
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
            line_of(&source, "query(sql`SELECT ${suppressedHost}"),
            line_of(&source, "query(sql`SELECT ${suppressedMutation}"),
        ],
        "{report}"
    );
}
