use serde_json::Value;
use std::path::PathBuf;
use std::process::Output;

const RULE: &str = "test-no-live-repository-scan";

/// Each root owns `POISON_DIR` once, then something this analysis cannot name
/// writes it too; `control` has no second write and is accepted.
const POISONED: [&str; 47] = [
    "global-constructor",
    "global-proto",
    "process-builtin",
    "process-main-module",
    "reexport-named",
    "reexport-star",
    "reexport-default",
    "ctor-function",
    "ctor-async",
    "ctor-prototype",
    "ctor-computed",
    "ctor-destructure",
    "ctor-reflect",
    "ctor-descriptor",
    "ctor-var-key",
    "global-spyon",
    "global-alias",
    "global-define",
    "load-actual-process",
    "load-computed",
    "load-create-require",
    "load-eval",
    "destructure-process",
    "import-env",
    "globalthis",
    "computed-env",
    "object-target",
    "array-target",
    "alias-assign",
    "bare-argument",
    "update-target",
    "unstub-all",
    "unstub-config",
    "setup-file",
    "import-process",
    "require-process",
    "forof-target",
    "stub-alias",
    "stub-destructure",
    "stub-computed",
    "unstub-computed",
    "vi-template",
    "vi-variable",
    "vi-stub-global",
    "stub-global-import",
    "unstub-computed-key",
    "unstub-workspace-json",
];

/// Each root owns a default parameter through one direct caller, then the
/// function escapes where the call graph cannot see; `param-control` is accepted.
const ESCAPED: [&str; 9] = [
    "escape-alias",
    "escape-alias-actual",
    "escape-default",
    "escape-export-default",
    "escape-require",
    "escape-dynamic",
    "escape-namespace",
    "escape-import-call",
    "escape-actual",
];

fn root(kind: &str) -> PathBuf {
    crate::check_json_snapshot::rules_path(RULE, kind, "")
}

fn check_in(kind: &str, config: &str) -> (Output, Value) {
    let root = root(kind);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, report)
}

fn located(items: &Value) -> Vec<String> {
    let mut found: Vec<String> = items
        .as_array()
        .map(|items| items.as_slice())
        .unwrap_or_default()
        .iter()
        .filter(|item| item["rule"] == RULE)
        .map(|item| format!("{}:{}", item["file"].as_str().unwrap(), item["line"]))
        .collect();
    found.sort();
    found
}

#[test]
fn provenance_json_snapshot() {
    let (provenance, bulk) = (root("provenance"), root("bulk"));
    let mut cases: Vec<(&str, PathBuf, PathBuf)> = vec![
        (
            "temp_factory_identity",
            root("temp-factory-identity"),
            root("temp-factory-identity").join(".no-mistakes.yml"),
        ),
        (
            "default",
            provenance.clone(),
            provenance.join(".no-mistakes.yml"),
        ),
        (
            "without_runtime_assets",
            provenance.clone(),
            provenance.join("no-assets.yml"),
        ),
        (
            "bulk_env_write",
            bulk.clone(),
            bulk.join(".no-mistakes.yml"),
        ),
    ];
    for name in ["control", "param-control", "vi-control"]
        .into_iter()
        .chain(POISONED)
        .chain(ESCAPED)
    {
        let poison = root(&format!("poison/{name}"));
        cases.push((name, poison.clone(), poison.join(".no-mistakes.yml")));
    }
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_provenance",
        &cases,
    );
}

#[test]
fn node_temp_factories_need_their_original_binding() {
    let (_, report) = check_in("temp-factory-identity", ".no-mistakes.yml");
    let mut findings = located(&report["rules"]);
    findings.extend(located(&report["advisories"]));
    findings.sort();
    for line in [24, 27, 30, 31, 32] {
        let expected = format!("tests/scan.test.mts:{line}");
        assert!(findings.contains(&expected), "{expected}: {findings:?}");
    }
    for line in 35..=40 {
        let unexpected = format!("tests/scan.test.mts:{line}");
        assert!(
            !findings.contains(&unexpected),
            "{unexpected}: {findings:?}"
        );
    }
    assert!(
        findings.contains(&"tests/require.test.cts:15".to_string()),
        "{findings:?}"
    );
    for line in [11, 12] {
        let unexpected = format!("tests/require.test.cts:{line}");
        assert!(
            !findings.contains(&unexpected),
            "{unexpected}: {findings:?}"
        );
    }
    for line in [14, 17] {
        let expected = format!("tests/aliases.test.mts:{line}");
        assert!(findings.contains(&expected), "{expected}: {findings:?}");
    }
    assert!(
        !findings.contains(&"tests/aliases.test.mts:20".to_string()),
        "{findings:?}"
    );
    for line in [13, 14, 15] {
        let expected = format!("tests/mutation.test.mts:{line}");
        assert!(findings.contains(&expected), "{expected}: {findings:?}");
    }
    assert!(
        findings.contains(&"tests/require-mutation.test.cts:11".to_string()),
        "{findings:?}"
    );
    assert!(
        findings.contains(&"tests/computed-mutation.test.cts:11".to_string()),
        "{findings:?}"
    );
    assert!(
        findings.contains(&"tests/fake-spy.test.mts:10".to_string()),
        "{findings:?}"
    );
    assert!(
        findings.contains(&"tests/factory-spy.test.mts:9".to_string()),
        "{findings:?}"
    );
    assert!(
        !findings.contains(&"tests/spy.test.mts:11".to_string()),
        "{findings:?}"
    );
    assert!(
        !findings.contains(&"tests/unrelated-write.test.cts:10".to_string()),
        "{findings:?}"
    );
}

#[test]
fn owned_env_and_arguments_resolve_but_unproven_provenance_is_kept() {
    let (_, report) = check_in("provenance", ".no-mistakes.yml");
    let advisories = located(&report["advisories"]);
    let live = located(&report["rules"]);
    let reported: Vec<_> = advisories.iter().chain(&live).cloned().collect();
    for location in [
        "tests/pass-through.test.ts:14",
        "tests/pass-through.test.ts:29",
    ] {
        assert!(live.contains(&location.to_string()), "{location}: {live:?}");
    }
    // Accepted: an owned env root through a getter and join, a hook, an owned
    // argument for a defaulted parameter, and exact spread-forwarding mocks.
    for resolved in [
        "src/env-owned.ts",
        "src/env-hook.ts",
        "src/env-chain.ts",
        "src/param-sync.ts",
        "tests/pass-through.test.ts:8",
        "tests/pass-through.test.ts:10",
    ] {
        assert!(
            reported.iter().all(|f| !f.contains(resolved)),
            "{resolved}: {advisories:?}"
        );
    }
    // Kept: late, conditional, foreign, and unwritten env; omitted, mixed,
    // rebound, and nested parameters; and mocks that change the arguments.
    for kept in [
        "src/env-late.ts:6",
        "src/env-captured.ts:7",
        "src/env-toplevel.ts:3",
        "src/env-cond-hook.ts:6",
        "src/env-conditional.ts:6",
        "src/env-mixed.ts:6",
        "src/env-unwritten.ts:6",
        "src/param-omitted.ts:6",
        "src/param-mixed.ts:4",
        "src/param-reassigned.ts:5",
        "src/param-nested.ts:4",
        "tests/pass-through.test.ts:14",
        "tests/pass-through.test.ts:15",
        "tests/pass-through.test.ts:16",
        "tests/pass-through.test.ts:17",
        "tests/pass-through.test.ts:19",
        "tests/pass-through.test.ts:23",
        "tests/pass-through.test.ts:24",
        // The wrapper mutates the forwarded arguments (index write, splice,
        // length/push, mutation inside a nested closure).
        "tests/pass-through.test.ts:29",
        "tests/pass-through.test.ts:33",
        "tests/pass-through.test.ts:38",
        "tests/pass-through.test.ts:45",
        // A function also passed as a value, or called through `.call`, has
        // callers whose arguments are unknown.
        "src/param-escape.ts:4",
        "src/param-call.ts:4",
        // An owned root does not cover a path that climbs out of it with `..`.
        "src/env-climb.ts:8",
        "src/env-write-climb.ts:7",
        "src/param-climb.ts:6",
    ] {
        assert!(
            reported.contains(&kept.to_string()),
            "{kept}: {advisories:?}"
        );
    }
    // A bulk write to process.env poisons the owned assignment next to it.
    let (_, report) = check_in("bulk", ".no-mistakes.yml");
    assert_eq!(
        located(&report["advisories"]),
        ["src/env-bulk.ts:6"],
        "{report}"
    );
}

#[test]
fn runtime_assets_are_listed_separately_and_near_misses_stay_reported() {
    let (_, report) = check_in("provenance", ".no-mistakes.yml");
    let findings = located(&report["rules"]);
    for live in [
        "src/near-miss.ts:5",
        "src/asset-parent.ts:5",
        "src/asset-escape.ts:5",
        "src/source-tree.ts:4",
    ] {
        assert!(findings.contains(&live.to_string()), "{live}: {findings:?}");
    }
    assert!(
        findings.iter().all(|f| !f.contains("migration-status")),
        "{findings:?}"
    );
    let assets: Vec<&Value> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["target"] == "runtime asset")
        .collect();
    // The suppressed read adds no second entry.
    assert_eq!(assets.len(), 1, "{report}");
    let message = assets[0]["message"].as_str().unwrap();
    assert!(message.contains("shipped migration files"), "{message}");
    // Without the declaration the same read is a live scan.
    let (_, report) = check_in("provenance", "no-assets.yml");
    assert!(located(&report["rules"]).contains(&"src/migration-status.ts:5".to_string()));
}

#[test]
fn runtime_asset_declarations_need_a_reason() {
    let (output, _) = check_in("provenance", "invalid-assets.yml");
    let text = String::from_utf8_lossy(&output.stderr).to_string()
        + &String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success());
    assert!(text.contains("runtimeAssets entries need"), "{text}");
}

#[test]
fn unnamed_env_writes_poison_an_owned_variable() {
    let (_, report) = check_in("poison/control", ".no-mistakes.yml");
    assert_eq!(
        located(&report["advisories"]),
        Vec::<String>::new(),
        "{report}"
    );
    for name in POISONED {
        let (_, report) = check_in(&format!("poison/{name}"), ".no-mistakes.yml");
        assert_eq!(
            located(&report["advisories"]),
            ["src/dir.ts:6"],
            "{name}: {report}"
        );
    }
}

#[test]
fn escaped_default_parameter_functions_keep_the_finding() {
    let (_, report) = check_in("poison/param-control", ".no-mistakes.yml");
    assert_eq!(
        located(&report["advisories"]),
        Vec::<String>::new(),
        "{report}"
    );
    for name in ESCAPED {
        let (_, report) = check_in(&format!("poison/{name}"), ".no-mistakes.yml");
        assert_eq!(
            located(&report["advisories"]),
            ["src/dir.ts:8"],
            "{name}: {report}"
        );
    }
}
