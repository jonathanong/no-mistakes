use std::path::PathBuf;

#[test]
fn queue_wait_wrappers_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-wrappers/fixture");
    let cases = ["legacy-unconfigured", "exact", "patterns", "ban"]
        .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_wait_wrappers",
        &cases,
        &[],
        &[],
    );
    let exact_calls = [
        ("api/routes/bare-reference.ts", 3),
        ("api/routes/callback-reference.ts", 3),
        ("api/routes/computed-string.ts", 2),
        ("api/routes/computed-template.ts", 2),
        ("api/routes/const-alias-chain.ts", 4),
        ("api/routes/destructured-const.ts", 3),
        ("api/routes/direct-const-alias.ts", 3),
        ("api/routes/exact-bare.ts", 2),
        ("api/routes/exact-member.ts", 2),
        ("api/routes/import-alias.ts", 2),
    ];
    let declarations = [
        ("api/routes/declaration-arrow.ts", 1),
        ("api/routes/declaration-expression.ts", 1),
        ("api/routes/declaration-function.ts", 1),
    ];
    for (case, _, _) in cases {
        assert_eq!(reports[case]["exitCode"], 1);
        let report = &reports[case]["report"];
        assert!(report["warnings"].as_array().unwrap().is_empty());
        let mut actual: Vec<_> = report["queues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                (
                    finding["file"].as_str().unwrap(),
                    finding["line"].as_u64().unwrap(),
                    finding["kind"].as_str().unwrap(),
                )
            })
            .collect();
        let mut expected = vec![("api/routes/legacy-member.ts", 2, "enqueue-and-wait")];
        if case != "legacy-unconfigured" {
            expected.extend(exact_calls.map(|(file, line)| (file, line, "enqueue-and-wait")));
        }
        if matches!(case, "patterns" | "ban") {
            expected.push(("api/routes/pattern-bare.ts", 2, "enqueue-and-wait"));
        }
        if case == "ban" {
            expected.extend(
                declarations.map(|(file, line)| (file, line, "enqueue-and-wait-declaration")),
            );
        }
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected, "{case}");
    }
}
