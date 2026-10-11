use super::{check, PathBuf};

#[test]
fn mutable_builder_later_argument_offsets_are_visible_even_when_dynamic_sql_is_ignored() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/executor-argument-fail");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// findings: ")
                .map(|(_, target)| (index as u64 + 1, target))
        })
        .collect::<Vec<_>>();
    for (case, ignore) in [
        ("executor-argument-fail", false),
        ("executor-argument-ignore", true),
    ] {
        let report = check("postgres-no-offset", case);
        let actual = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                (
                    finding["line"].as_u64().unwrap(),
                    finding["target"]
                        .as_str()
                        .unwrap()
                        .split('#')
                        .next()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let expected = expected
            .iter()
            .copied()
            .filter(|(_, target)| !ignore || *target != "unanalyzable")
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{case}: {report}");
    }
}
