use super::judge::{scan_script, HitKind};
use super::options::{self, Options};
use super::{check, check_with_files, diagnostics_with_files_and_sources, RULE_ID};
use crate::codebase::ts_source::discover_files;
use crate::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::PathBuf;

fn compile_yaml(yaml: &str) -> options::CompiledOptions {
    let opts: Options = serde_yaml::from_str(yaml).expect("valid options yaml");
    options::compile(opts).expect("valid compiled options")
}

fn default_policy() -> options::CompiledOptions {
    compile_yaml(
        r#"
commands:
  - command: [gh, api]
    valueFlags: [-f, -F, --field, --raw-field, -X, --method, -q, --jq]
    whenAnyArgs: [-f, -F, --field, --raw-field]
    requireAnyArgs: [--method, -X]
    message: "Specify --method or -X with field flags."
  - command: [gh, api]
    valueFlags: [-f, -F, --field, --raw-field, -X, --method, -q, --jq]
    forbidArgPatterns:
      - pattern: '\?'
        scope: positional
    message: "Avoid query strings in gh api endpoints."
  - command: [git, commit]
    valueFlags: [-c, -C, --git-dir]
    requireArgs:
      - [-c, core.hooksPath=/dev/null]
    message: "Disable repository hooks on CI commits."
"#,
    )
}

#[test]
fn detects_required_and_forbidden_arguments_across_forms() {
    let compiled = default_policy();
    let hits = scan_script(
        r#"
gh api -f x=1 repos/my-org/my-repo/labels
gh api --field=x=1 repos/my-org/my-repo/labels
gh api -fx=1 repos/my-org/my-repo/labels
gh api "repos/my-org/my-repo/labels?per_page=100"
gh api -- "repos/my-org/my-repo/labels?per_page=100"
git commit -m "chore: update"
git -C subdir commit -m "chore: update"
git commit -- -c core.hooksPath=/dev/null
"#,
        &compiled,
    );
    assert_eq!(hits.len(), 8);
    assert!(matches!(hits[0].kind, HitKind::MissingAnyRequired { .. }));
    assert!(matches!(hits[1].kind, HitKind::MissingAnyRequired { .. }));
    assert!(matches!(hits[2].kind, HitKind::MissingAnyRequired { .. }));
    assert!(matches!(hits[3].kind, HitKind::ForbiddenArg { .. }));
    assert!(matches!(hits[4].kind, HitKind::ForbiddenArg { .. }));
    assert!(matches!(hits[5].kind, HitKind::MissingRequired { .. }));
    assert!(matches!(hits[6].kind, HitKind::MissingRequired { .. }));
    assert!(matches!(hits[7].kind, HitKind::MissingRequired { .. }));
}

#[test]
fn accepts_compliant_attached_inline_and_wrapper_forms() {
    let compiled = default_policy();
    let hits = scan_script(
        r#"
gh api --method GET -f x=1 repos/my-org/my-repo/labels
gh api --method=GET -F per_page=100 repos/my-org/my-repo/labels
gh api -X GET -f x=1 repos/my-org/my-repo/labels
gh api -XGET -f x=1 repos/my-org/my-repo/labels
gh api repos/my-org/my-repo/labels
gh api --method GET -f "q=a?b" --jq '.x?' repos/my-org/my-repo/labels
git -c core.hooksPath=/dev/null commit -m "chore: update"
git -c=core.hooksPath=/dev/null commit -m "chore: update"
git -ccore.hooksPath=/dev/null commit -m "chore: update"
git -C sub -c core.hooksPath=/dev/null commit -m "chore: update"
git status
echo 'gh api -f x=1 repos/my-org/my-repo/labels'
echo '$(gh api -f x=1 repos/my-org/my-repo/labels)'
"#,
        &compiled,
    );
    assert!(hits.is_empty(), "expected no findings, got {}", hits.len());
}

#[test]
fn inspects_nested_shell_c_and_command_substitutions() {
    let compiled = default_policy();
    let hits = scan_script(
        r#"
OUT=$(gh api -f x=1 repos/my-org/my-repo/labels)
sh -ec 'git commit -m "nested"'
echo $(( 1 + 2 ))
"#,
        &compiled,
    );
    assert_eq!(hits.len(), 2, "{hits:?}");
    assert_eq!(hits[0].line, 2);
    assert!(matches!(hits[0].kind, HitKind::MissingAnyRequired { .. }));
    assert_eq!(hits[1].line, 3);
    assert!(matches!(hits[1].kind, HitKind::MissingRequired { .. }));
}

#[test]
fn abstains_on_dynamic_commands_eval_and_unterminated_syntax() {
    let compiled = default_policy();
    for script in [
        "$CMD api -f x=1 repos/my-org/my-repo/labels",
        "\"$CMD\" commit -m hi",
        "${{ inputs.cli }} api repos/my-org/my-repo/labels",
        "eval \"gh api -f x=1 repos/my-org/my-repo/labels\"",
        "bash -c \"$SCRIPT\"",
        "git \"$SUBCMD\" -m hi",
        "gh api $EXTRA_FLAGS repos/my-org/my-repo/labels",
        "git commit \"${GIT_FLAGS[@]}\" -m hi",
        "gh api \"unterminated",
        "cat <<EOF\nhello",
        "OUT=$(gh api",
    ] {
        let hits = scan_script(script, &compiled);
        assert!(
            !hits.is_empty()
                && hits
                    .iter()
                    .all(|hit| matches!(hit.kind, HitKind::Unanalyzable { .. })),
            "expected abstention for `{script}`"
        );
    }
}

#[test]
fn quoted_scalar_value_does_not_hide_missing_required_arg() {
    let compiled = default_policy();
    let hits = scan_script(
        r#"
git commit -m "$COMMIT_MSG"
gh api -f "title=${TITLE}" repos/my-org/my-repo/labels
"#,
        &compiled,
    );
    assert_eq!(hits.len(), 2);
    assert!(matches!(hits[0].kind, HitKind::MissingRequired { .. }));
    assert!(matches!(hits[1].kind, HitKind::MissingAnyRequired { .. }));
}

#[test]
fn infers_value_flag_from_two_token_required_pair() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [git, commit]
    requireArgs:
      - [-c, core.hooksPath=/dev/null]
"#,
    );
    let ok = scan_script("git -c core.hooksPath=/dev/null commit -m hi", &compiled);
    assert!(ok.is_empty());
    let bad = scan_script("git commit -m hi", &compiled);
    assert_eq!(bad.len(), 1);
    assert!(matches!(bad[0].kind, HitKind::MissingRequired { .. }));
}

#[test]
fn rejects_invalid_option_configurations() {
    for (yaml, needle) in [
        ("commands: [{ command: [] }]", "non-empty list"),
        (
            "commands: [{ command: ['gh api'], requireArgs: ['--method'] }]",
            "without whitespace",
        ),
        (
            "commands: [{ command: [gh], valueFlags: ['method'], requireArgs: ['--method'] }]",
            "valueFlags",
        ),
        (
            "commands: [{ command: [gh, api] }]",
            "at least one of requireArgs",
        ),
        (
            "commands: [{ command: [gh, api], forbidArgPatterns: [{ pattern: '[' }] }]",
            "invalid regex",
        ),
        (
            "commands: [{ command: [gh, api], requireArgs: [[]] }]",
            "non-empty argument strings",
        ),
    ] {
        let opts: Options = serde_yaml::from_str(yaml).expect("deserializes");
        let err = options::compile(opts)
            .err()
            .expect("expected compilation failure");
        assert!(
            format!("{err:#}").contains(needle),
            "expected `{needle}` in `{err:#}`"
        );
    }
}

#[test]
fn standalone_and_diagnostics_entrypoints_match_saved_fixtures() {
    let base = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(RULE_ID),
    );
    let config_path = base.join("configs/policy.yml");
    let reported_root = base.join("fixture/reported");
    let config: NoMistakesConfig = load_v2_config(&reported_root, Some(&config_path)).unwrap();
    let findings = check(&reported_root, &config).unwrap();
    assert_eq!(findings.len(), 7);
    let files = discover_files(&reported_root, &config.filesystem.skip_directories);
    assert_eq!(
        check_with_files(&reported_root, &config, &files).unwrap(),
        findings
    );

    let unparsable_root = base.join("fixture/unparsable");
    let unparsable_files = discover_files(&unparsable_root, &config.filesystem.skip_directories);
    let sources = crate::codebase::rules::source_store_for_files(&unparsable_files);
    let diag = diagnostics_with_files_and_sources(
        &unparsable_root,
        &config,
        &unparsable_files,
        &sources,
        false,
    )
    .unwrap();
    assert!(diag.findings.is_empty());
    assert_eq!(diag.advisories.len(), 4);
}
