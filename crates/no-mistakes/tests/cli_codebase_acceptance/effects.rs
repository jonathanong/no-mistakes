use super::common::{assert_success, fixture, run, run_in, run_json, stdout};

#[test]
fn effects_json_reports_reachable_call_sites() {
    let root = fixture("effects");
    let value = run_json(&root, &["effects", "valkey", "--entry", "app/server.ts"]);
    assert_eq!(value["kind"], "valkey");

    let sites = value["callSites"].as_array().unwrap();
    assert_eq!(sites.len(), 4);
    assert_eq!(value["byCategory"]["cache"], 2);
    assert_eq!(value["byCategory"]["pubsub"], 1);
    assert_eq!(value["byCategory"]["invalidation"], 1);

    // The unreachable file is never scanned.
    assert!(sites.iter().all(|s| s["file"] != "lib/unused.ts"));
}

#[test]
fn effects_typed_and_returned_members_preserve_category_filters() {
    let root =
        fixture("effects").join("../../../rules/query-reached-per-item/typed-client-fixture");
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let value = run_json(
        &root,
        &[
            "effects",
            "postgres",
            "--entry",
            "src/entry.mts",
            "--category",
            "transaction",
        ],
    );
    assert_eq!(value["callSites"].as_array().unwrap().len(), 5);
    assert_eq!(value["byCategory"]["transaction"], 5);
    assert!(value["callSites"]
        .as_array()
        .unwrap()
        .iter()
        .any(|site| site["callee"] == "writePool.connect().query"));
    let output = run_in(
        &root,
        &[
            "effects",
            "postgres",
            "--entry",
            "src/entry.mts",
            "--category",
            "absent",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("no functions for the requested categories"));
}

#[test]
fn effects_category_filter() {
    let root = fixture("effects");
    let value = run_json(
        &root,
        &[
            "effects",
            "valkey",
            "--entry",
            "app/server.ts",
            "--category",
            "pubsub",
        ],
    );
    let sites = value["callSites"].as_array().unwrap();
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0]["callee"], "createPublisher");
}

#[test]
fn effects_paths_format() {
    let root = fixture("effects");
    let root_arg = root.to_string_lossy();
    let output = run(&[
        "effects",
        "valkey",
        "--entry",
        "app/server.ts",
        "--root",
        root_arg.as_ref(),
        "--format",
        "paths",
    ]);
    assert_success(&output);
    assert!(stdout(&output).contains("lib/cache.ts"));
}

#[test]
fn effects_human_and_md_formats() {
    let root = fixture("effects");
    let human = run_in(&root, &["effects", "valkey", "--entry", "app/server.ts"]);
    assert_success(&human);
    assert!(stdout(&human).contains("ValkeyCache"));

    let md = run_in(
        &root,
        &[
            "effects",
            "valkey",
            "--entry",
            "app/server.ts",
            "--format",
            "md",
        ],
    );
    assert_success(&md);
    assert!(stdout(&md).contains("# effects `valkey`"));
}

#[test]
fn effects_yml_format() {
    let root = fixture("effects");
    let yml = run_in(
        &root,
        &[
            "effects",
            "valkey",
            "--entry",
            "app/server.ts",
            "--format",
            "yml",
        ],
    );
    assert_success(&yml);
    assert!(stdout(&yml).contains("kind: valkey"));
}

#[test]
fn effects_unknown_kind_fails() {
    let root = fixture("effects");
    let root_arg = root.to_string_lossy();
    let output = run(&[
        "effects",
        "bogus",
        "--entry",
        "app/server.ts",
        "--root",
        root_arg.as_ref(),
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown effects kind"));
}

#[test]
fn effects_cli_retains_inline_invocation_context() {
    let root = fixture("inline-call-context");
    for (name, expected) in [
        ("inline", 1),
        ("function", 1),
        ("default", 1),
        ("returned", 1),
        ("async", 0),
        ("generator", 0),
        ("returned-async", 0),
        ("after", 0),
        ("captured", 0),
        ("callback", 0),
        ("direct-escape", 0),
        ("bound-escape", 0),
        ("mixed-callback", 0),
        ("mixed-named-callback", 0),
        ("top-level", 1),
    ] {
        let entry = format!("src/{name}.mts");
        let report = run_json(&root, &["effects", "cache", "--entry", &entry]);
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}

#[test]
fn effects_namespace_types_cross_explicit_barrels() {
    let root =
        fixture("effects").join("../../../rules/query-reached-per-item/namespace-types-fixture");
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let value = run_json(&root, &["effects", "postgres", "--entry", "src/entry.mts"]);
    assert_eq!(value["callSites"].as_array().unwrap().len(), 3, "{value}");
}

#[test]
fn effects_cli_retains_cross_file_inline_receiver_timing() {
    let root = fixture("cross-file-inline-receiver");
    for (name, expected) in [
        ("earlier", 1),
        ("sibling", 1),
        ("reexported", 1),
        ("defaults", 1),
        ("mutated-default", 0),
        ("later", 0),
        ("prior", 0),
        ("other-context", 0),
        ("conflicting", 0),
        ("unknown", 0),
        ("escaped", 0),
    ] {
        let entry = format!("src/{name}.mts");
        let report = run_json(&root, &["effects", "cache", "--entry", &entry]);
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}

#[test]
fn effects_cli_respects_definite_enclosing_initialization() {
    let root = fixture("value-flow-receiver-evaluation-order");
    let report = run_json(
        &root,
        &["effects", "cache", "--entry", "src/enclosing-tdz.mts"],
    );
    let sites = report["callSites"].as_array().unwrap();
    assert_eq!(sites.len(), 6, "{report}");
    let callers = sites
        .iter()
        .map(|site| site["caller"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        callers,
        std::collections::BTreeSet::from([
            "initializedRun",
            "conditionalRun",
            "branchRun",
            "hoistedRun",
            "staticInvoke",
            "switchRun",
        ])
    );
    assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
}

#[test]
fn effects_cli_preserves_transparent_default_aliases_and_member_snapshots() {
    let root = fixture("inline-member-precision");
    for (name, expected) in [
        ("unique-helper-repeated", 0),
        ("unique-helper-snapshot", 1),
        ("unique-helper-setup", 1),
        ("alias-named", 1),
        ("alias-inline", 1),
        ("alias-chain", 1),
        ("alias-matching", 0),
        ("alias-dynamic", 0),
        ("alias-live", 0),
        ("alias-supplied-live", 0),
        ("alias-supplied-snapshot", 1),
        ("alias-supplied-nested-live", 0),
        ("alias-snapshot", 1),
        ("alias-inline-live", 0),
        ("alias-inline-snapshot", 1),
        ("alias-inline-snapshot-owned", 1),
        ("inline-snapshot-owned", 1),
        ("default-snapshot-owned", 1),
        ("default-object-snapshot-owned", 1),
        ("default-snapshot-write-first", 0),
        ("default-snapshot-deferred-arrow", 0),
        ("default-snapshot-deferred-function", 0),
        ("default-snapshot-deferred-method", 0),
        ("default-snapshot-nested", 0),
        ("default-snapshot-loop", 0),
        ("default-snapshot-field", 0),
        ("default-snapshot-accessor", 0),
        ("default-snapshot-async", 0),
        ("default-snapshot-reentrant", 0),
        ("default-computed-arrow", 0),
        ("default-computed-function", 0),
        ("default-computed-method", 0),
        ("default-computed-class", 0),
        ("default-computed-parameter", 0),
        ("default-computed-key", 1),
        ("default-computed-class-key", 1),
        ("default-computed-restored", 1),
        ("module-computed-arrow", 0),
        ("inline-snapshot-owned-write-first", 0),
        ("inline-snapshot-repeated-for", 0),
        ("inline-snapshot-repeated-header", 0),
        ("inline-snapshot-repeated-function", 0),
        ("inline-snapshot-repeated-while", 0),
        ("inline-snapshot-repeated-do", 0),
        ("inline-snapshot-repeated-field", 0),
        ("inline-snapshot-repeated-nested-field", 0),
        ("inline-snapshot-repeated-accessor", 0),
        ("alias-captured", 0),
        ("alias-async", 0),
    ] {
        let entry = format!("src/{name}.mts");
        let report = run_json(&root, &["effects", "cache", "--entry", &entry]);
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}

#[test]
fn effects_alternatives_json_and_text_keep_one_physical_occurrence() {
    let root = fixture("effects-alternatives");
    for (projection, category) in [
        ("all", None),
        ("read", Some("read")),
        ("write", Some("write")),
    ] {
        let mut args = vec!["effects", "database", "--entry", "src/entry.mts"];
        if let Some(category) = category {
            args.extend(["--category", category]);
        }
        let actual = run_json(&root, &args);
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("expected-{projection}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
    for format in ["human", "md"] {
        let output = run_in(
            &root,
            &[
                "effects",
                "database",
                "--entry",
                "src/entry.mts",
                "--format",
                format,
            ],
        );
        assert_success(&output);
        let text = stdout(&output);
        assert_eq!(text.matches("src/entry.mts:4").count(), 1);
        assert!(text.contains("alternatives=[primary.read (read), replica.write (write)]"));
    }
}

#[test]
fn effects_cli_distinguishes_loop_setup_from_repeated_positions() {
    let root = fixture("inline-member-precision");
    for (name, expected) in [
        ("loop-setup-annex.cjs", 0),
        ("loop-setup-async.mts", 0),
        ("loop-setup-for-declaration.mts", 1),
        ("loop-setup-for-expression.mts", 1),
        ("loop-setup-for-update.mts", 0),
        ("loop-setup-function.mts", 0),
        ("loop-setup-in-body.mts", 0),
        ("loop-setup-in-default.mts", 0),
        ("loop-setup-in-rhs.mts", 1),
        ("loop-setup-in-target.mts", 0),
        ("loop-setup-nested-for.mts", 0),
        ("loop-setup-nested-in.mts", 0),
        ("loop-setup-nested-of.mts", 0),
        ("loop-setup-of-body.mts", 0),
        ("loop-setup-of-default.mts", 0),
        ("loop-setup-of-rhs.mts", 1),
        ("loop-setup-of-target.mts", 0),
    ] {
        let entry = format!("src/{name}");
        let report = run_json(&root, &["effects", "cache", "--entry", &entry]);
        let sites = report["callSites"].as_array().unwrap();
        assert_eq!(sites.len(), expected, "{name}: {report}");
        assert!(sites.iter().all(|site| site["callee"] == "primary.get"));
    }
}

#[test]
fn effects_cli_invalidates_one_time_lexical_writes_before_a_loop() {
    let root = fixture("value-flow-lexical-member-replacement");
    let report = run_json(&root, &["effects", "cache", "--entry", "src/entry.mts"]);
    let sites = report["callSites"].as_array().unwrap();
    let actual: Vec<_> = sites
        .iter()
        .map(|site| {
            (
                site["line"].as_u64().unwrap(),
                site["callee"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            (3, "primary.get"),
            (8, "primary.get"),
            (9, "primary.info"),
            (10, "untouched.get")
        ],
        "{report}"
    );
    let report = run_json(&root, &["effects", "cache", "--entry", "src/minimal.mts"]);
    assert!(
        report["callSites"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let report = run_json(&root, &["effects", "cache", "--entry", "src/after.mts"]);
    assert_eq!(report["callSites"].as_array().unwrap().len(), 1, "{report}");
}

#[test]
fn effects_project_uses_the_same_executors_as_postgres_rules() {
    let root = fixture("postgres-project-scope");
    let value = run_json(&root, &["effects", "postgres", "--entry", "src/query.ts"]);
    let callees: Vec<_> = value["callSites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["callee"].as_str().unwrap())
        .collect();
    assert!(callees.contains(&"beginTransaction"), "{value}");
    assert!(callees.contains(&"TransactionQuery"), "{value}");
    assert!(callees.contains(&"writePool.connect().query"), "{value}");

    let config = root.join(".no-mistakes.yml");
    let output = run(&[
        "check",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--json",
    ]);
    let body = stdout(&output);
    assert!(!output.status.success(), "{body}");
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    let files: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "postgres-no-offset")
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    assert!(files.contains(&5), "{report}");
    assert!(files.contains(&6), "{report}");
}

#[test]
fn lexical_replaced_members_reach_unknown_call_rule_findings() {
    let root = fixture("value-flow-lexical-member-replacement");
    let output = run_in(
        &root,
        &[
            "check",
            "--root",
            root.to_str().unwrap(),
            "--config",
            "rules.yml",
            "--format",
            "json",
        ],
    );
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let actual: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            (7, "unknown call"),
            (13, "unknown call"),
            (15, "unknown call")
        ],
        "{report}"
    );
}
