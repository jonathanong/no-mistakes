use super::{check, check_with_suppressed};
use std::path::PathBuf;

fn rows() -> Vec<(usize, String)> {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/variants/captured-identities-fail/src/query.ts",
    ))
    .unwrap()
    .lines()
    .enumerate()
    .map(|(index, line)| (index + 1, line.to_owned()))
    .collect()
}

#[test]
fn independent_final_builder_effects_keep_finite_checks_under_fail_and_ignore() {
    for (case, fail) in [
        ("captured-identities-fail", true),
        ("captured-identities-ignore", false),
    ] {
        let mut expected = rows()
            .into_iter()
            .filter_map(|(line, source)| {
                if source.contains("? sql`id FROM pages") {
                    Some((line, "offset"))
                } else if fail && source.starts_with("query(") && source.ends_with("// opaque") {
                    Some((line, "unanalyzable"))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        expected.sort_unstable();
        let report = check("postgres-no-offset", case);
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
fn captured_identity_filtering_keeps_fragment_and_host_suppression_accounting() {
    let expected = rows()
        .into_iter()
        .filter_map(|(line, source)| {
            (source.contains("? sql`id FROM suppressed_pages")
                || source.ends_with("// suppressed opaque"))
            .then_some(line)
        })
        .collect::<Vec<_>>();
    let report = check_with_suppressed("postgres-no-offset", "captured-identities-fail", true);
    let actual = report["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["directive"]["kind"], "nextLine", "{report}");
            finding["line"].as_u64().unwrap() as usize
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{report}");
}
