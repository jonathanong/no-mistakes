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
