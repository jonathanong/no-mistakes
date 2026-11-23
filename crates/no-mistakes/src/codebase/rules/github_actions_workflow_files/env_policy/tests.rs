use super::{check_with_files, compile, lines, scan, Options, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;
use std::path::PathBuf;

fn rule_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-env-policy")
}

fn config(name: &str) -> NoMistakesConfig {
    let path = rule_root().join("configs").join(name);
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn violations_report_every_scope() {
    let root = rule_root().join("fixture/violations");
    let files = [
        root.join(".github/workflows/ci.yml"),
        root.join(".github/actions/setup/action.yml"),
    ];
    let found = check_with_files(&root, &config("policy.yml"), &files).unwrap();
    let lines: Vec<_> = found.iter().map(|f| (f.file.as_str(), f.line)).collect();
    assert_eq!(
        lines,
        [
            (".github/actions/setup/action.yml", 9),
            (".github/workflows/ci.yml", 4),
            (".github/workflows/ci.yml", 10),
            (".github/workflows/ci.yml", 15),
            (".github/workflows/ci.yml", 20),
        ]
    );
    assert!(found.iter().all(|f| f.rule == RULE_ID));
}

#[test]
fn pass_and_empty_report_nothing() {
    let pass = rule_root().join("fixture/pass");
    let file = pass.join(".github/workflows/ci.yml");
    assert!(
        check_with_files(&pass, &config("policy.yml"), std::slice::from_ref(&file))
            .unwrap()
            .is_empty()
    );
    assert!(check_with_files(&pass, &config("empty.yml"), &[file])
        .unwrap()
        .is_empty());
}

#[test]
fn conflicting_options_are_a_configuration_error() {
    let opts: Options = serde_yaml::from_str("required: {A: x}\nforbidden: [A]").unwrap();
    assert!(compile(&opts).is_err());
    let opts: Options = serde_yaml::from_str("required: {A: [x]}").unwrap();
    assert!(compile(&opts).is_err());
}

#[test]
fn non_scalar_value_is_a_mismatch_and_numbers_compare_as_text() {
    let opts: Options = serde_yaml::from_str("required: {LEVEL: 1}").unwrap();
    let policy = compile(&opts).unwrap();
    let source = "env:\n  LEVEL: 1\njobs:\n  a:\n    env: {LEVEL: [x]}\n";
    let value: Value = serde_yaml::from_str(source).unwrap();
    let found = scan::check_parsed("w.yml", source, &value, &policy);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].line, 5);
}

#[test]
fn locate_handles_same_indent_sequences() {
    use lines::{locate, Seg};
    let source = "jobs:\n  a:\n    steps:\n    - run: x\n    - run: y\n      env:\n        K: v\n";
    let at = [
        Seg::Key("jobs".into()),
        Seg::Key("a".into()),
        Seg::Key("steps".into()),
        Seg::Idx(1),
        Seg::Key("env".into()),
        Seg::Key("K".into()),
    ];
    assert_eq!(locate(source, &at), 7);
}

#[test]
fn locate_matches_quoted_keys() {
    use lines::{locate, Seg};
    let source = "\"env\":\n  'POLICY': wrong\n";
    let at = [Seg::Key("env".into()), Seg::Key("POLICY".into())];
    assert_eq!(locate(source, &at), 2);
}

#[test]
fn invalid_yaml_and_file_directive_report_nothing() {
    for case in ["invalid", "disabled-file"] {
        let root = rule_root().join("fixture").join(case);
        let file = root.join(".github/workflows/ci.yml");
        let found = check_with_files(&root, &config("policy.yml"), &[file]).unwrap();
        assert!(found.is_empty(), "{case}: {found:?}");
    }
}

#[test]
fn odd_shapes_are_skipped_or_fall_back_to_the_parent_line() {
    let opts: Options = serde_yaml::from_str("required: {A: ''}\nforbidden: [B]").unwrap();
    let policy = compile(&opts).unwrap();
    let source = "env: [a]\n1: x\njobs:\n  1:\n    env: {B: v}\n  s: skip\n  t:\n    env: {B: v, 2: x, A: z}\n    steps:\n      - skip\n      - env: {A: ''}\n\"q: [\n";
    let value: Value = serde_yaml::from_str(source).unwrap_or(Value::Null);
    assert!(scan::check_parsed("w.yml", source, &value, &policy).is_empty());
    let source = "jobs:\n  1:\n    env: {B: v}\n  s: skip\n  t:\n    env: {B: v, 2: x, A: z}\n    steps:\n      - skip\n      - env: {A: ''}\nenv: [a]\n";
    let value: Value = serde_yaml::from_str(source).unwrap();
    let found = scan::check_parsed("w.yml", source, &value, &policy);
    assert_eq!(found.iter().map(|f| f.line).collect::<Vec<_>>(), [6, 6]);
}

#[test]
fn composite_shapes_and_unterminated_quotes() {
    let opts: Options = serde_yaml::from_str("forbidden: [B]").unwrap();
    let policy = compile(&opts).unwrap();
    for source in [
        "runs:\n  using: node20\n  steps:\n    - env: {B: v}\n",
        "runs:\n  using: composite\n  steps: none\n",
        "runs: none\n",
    ] {
        let value: Value = serde_yaml::from_str(source).unwrap();
        assert!(scan::check_parsed("a.yml", source, &value, &policy).is_empty());
    }
    use lines::{locate, Seg};
    assert_eq!(locate("\"env: x\n", &[Seg::Key("env".into())]), 1);
    assert_eq!(locate("", &[Seg::Key("env".into())]), 1);
    assert_eq!(locate("a: 1\n", &[Seg::Key("b".into()), Seg::Idx(0)]), 1);
}

#[test]
fn explicit_include_missing_files_and_scalar_forms() {
    let root = rule_root().join("fixture/violations");
    let file = root.join(".github/workflows/ci.yml");
    let found = check_with_files(&root, &config("include.yml"), &[file]).unwrap();
    assert_eq!(found.len(), 2, "{found:?}");
    let gone = root.join(".github/workflows/gone.yml");
    assert!(check_with_files(&root, &config("include.yml"), &[gone])
        .unwrap()
        .is_empty());
    let opts: Options = serde_yaml::from_str("required: {FLAG: true, EMPTY: null, N: 1}").unwrap();
    let policy = compile(&opts).unwrap();
    let source = "env:\n  FLAG: true\n  EMPTY:\n  N: 1\n";
    let value: Value = serde_yaml::from_str(source).unwrap();
    assert!(scan::check_parsed("w.yml", source, &value, &policy).is_empty());
    use lines::{locate, Seg};
    let at = [Seg::Key("steps".into()), Seg::Idx(1)];
    assert_eq!(locate("steps:\n  -\n  - run: x\n", &at), 3);
}
