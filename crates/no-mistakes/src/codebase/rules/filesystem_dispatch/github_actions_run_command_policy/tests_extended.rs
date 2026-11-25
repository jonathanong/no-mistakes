use super::judge::{scan_script, HitKind};
use super::options::{self, Options};
use super::{collect_config_errors, diagnostics_with_files_and_sources, RULE_ID};
use crate::codebase::ts_source::discover_files;
use crate::config::v2::load_v2_config;
use std::path::PathBuf;

fn compile_yaml(yaml: &str) -> options::CompiledOptions {
    let opts: Options = serde_yaml::from_str(yaml).expect("valid options yaml");
    options::compile(opts).expect("valid compiled options")
}

fn gh_api_policy() -> options::CompiledOptions {
    compile_yaml(
        r#"
commands:
  - command: [gh, api]
    valueFlags: [-f, -F, --field, --raw-field, -X, --method, -H, --header, -q, --jq]
    whenAnyArgs: [-f, -F, --field, --raw-field]
    unlessAnyArgs: [graphql]
    requireAnyArgs: [--method, -X]
    message: "Specify --method or -X with field flags."
  - command: [gh, api]
    valueFlags: [-f, -F, --field, --raw-field, -X, --method, -H, --header, -q, --jq]
    forbidArgPatterns:
      - pattern: '[?]'
        scope: positional
    message: "Avoid query strings in gh api endpoints."
"#,
    )
}

#[test]
fn unless_any_args_exempts_graphql_and_abstains_on_ambiguous_dynamic_endpoint() {
    let compiled = gh_api_policy();
    let ok = scan_script(
        r#"
gh api graphql -f query='query { viewer { login } }'
gh api -X GET -f x=1 "$ENDPOINT"
gh api --method=GET -F per_page=100 "repos/${OWNER}/${REPO}/labels"
"#,
        &compiled,
    );
    assert!(ok.is_empty(), "expected no hits, got {ok:?}");

    let definite = scan_script(
        r#"
gh api -f x=1 "repos/${OWNER}/${REPO}/labels"
gh api -f x=1 "${OWNER}/${REPO}/labels"
gh api -f x=1 -q graphql repos/my-org/my-repo/labels
gh api "graphql?foo=bar" -f query='query { viewer { login } }'
"#,
        &compiled,
    );
    assert_eq!(definite.len(), 5, "{definite:?}");
    assert!(matches!(
        definite[0].kind,
        HitKind::MissingAnyRequired { .. }
    ));
    assert!(matches!(
        definite[1].kind,
        HitKind::MissingAnyRequired { .. }
    ));
    assert!(matches!(
        definite[2].kind,
        HitKind::MissingAnyRequired { .. }
    ));
    assert!(matches!(
        definite[3].kind,
        HitKind::MissingAnyRequired { .. }
    ));
    assert!(matches!(definite[4].kind, HitKind::ForbiddenArg { .. }));

    for dynamic in [
        r#"gh api -f x=1 "$ENDPOINT""#,
        r#"gh api -f x=1 "${ENDPOINT}""#,
        r#"gh api -f x=1 "graphql${SUFFIX}""#,
        r#"gh api -f x=1 "${PREFIX}graphql""#,
        r#"gh api -f x=1 "g${MID}l""#,
        r#"gh api -f x=1 "$(get_endpoint)""#,
        r#"gh api -f x=1 `get_endpoint`"#,
        r#"gh api -f x=1 "${{ inputs.endpoint }}""#,
    ] {
        let hits = scan_script(dynamic, &compiled);
        assert_eq!(
            hits.len(),
            1,
            "expected 1 advisory for `{dynamic}`, got {hits:?}"
        );
        assert!(
            matches!(hits[0].kind, HitKind::Unanalyzable { .. }),
            "expected Unanalyzable for `{dynamic}`, got {hits:?}"
        );
    }
}

#[test]
fn flag_matching_skips_consumed_values_and_rejects_empty_required_values() {
    let compiled = gh_api_policy();
    let bad = scan_script(
        r#"
gh api -f x=1 -H "-X: custom" repos/my-org/my-repo/labels
gh api --method= -f x=1 repos/my-org/my-repo/labels
gh api -X= -f x=1 repos/my-org/my-repo/labels
gh api -X "" -f x=1 repos/my-org/my-repo/labels
gh api -f x=1 repos/my-org/my-repo/labels -X
"#,
        &compiled,
    );
    assert_eq!(bad.len(), 5, "{bad:?}");
    assert!(bad
        .iter()
        .all(|hit| matches!(hit.kind, HitKind::MissingAnyRequired { .. })));

    let dynamic_methods = scan_script(
        r#"
gh api --method "$METHOD" -f x=1 repos/my-org/my-repo/labels
gh api --method="$METHOD" -f x=1 repos/my-org/my-repo/labels
gh api -X="$METHOD" -f x=1 repos/my-org/my-repo/labels
gh api -X"$METHOD" -f x=1 repos/my-org/my-repo/labels
gh api --method="${{ inputs.method }}" -f x=1 repos/my-org/my-repo/labels
"#,
        &compiled,
    );
    assert_eq!(dynamic_methods.len(), 5, "{dynamic_methods:?}");
    assert!(dynamic_methods
        .iter()
        .all(|hit| matches!(hit.kind, HitKind::Unanalyzable { .. })));

    let ok = scan_script(
        r#"
gh api --jq -f repos/my-org/my-repo/labels
gh api -X=GET -f x=1 repos/my-org/my-repo/labels
"#,
        &compiled,
    );
    assert!(ok.is_empty(), "{ok:?}");
}

#[test]
fn validates_shell_files_and_collects_config_errors_early() {
    for (yaml, needle) in [
        ("shellFiles: ['']", "options.shellFiles[0]"),
        ("shellFiles: ['[unclosed']", "options.shellFiles"),
        ("include: ['  ']", "options.include[0]"),
        (
            "commands: [{ command: [gh, api], unlessAnyArgs: [[]], requireArgs: ['-X'] }]",
            "unlessAnyArgs",
        ),
    ] {
        let opts: Options = serde_yaml::from_str(yaml).unwrap();
        let err = options::compile(opts)
            .err()
            .expect("expected compile error");
        assert!(format!("{err:#}").contains(needle), "{err:#}");
    }

    let base = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(RULE_ID),
    );
    let root = base.join("fixture/shell-scripts");
    let cfg_path = base.join("configs/invalid-shell-glob.yml");
    let config = load_v2_config(&root, Some(&cfg_path)).unwrap();
    let mut errors = Vec::new();
    collect_config_errors(&root, Some(&cfg_path), &config, &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("options.shellFiles"), "{errors:?}");

    let policy_cfg = base.join("configs/policy-shell.yml");
    let valid_config = load_v2_config(&root, Some(&policy_cfg)).unwrap();
    let files = discover_files(&root, &valid_config.filesystem.skip_directories);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let diag =
        diagnostics_with_files_and_sources(&root, &valid_config, &files, &sources, false).unwrap();
    assert!(!diag.findings.is_empty());
    assert!(!diag.advisories.is_empty());
}

#[test]
fn dynamic_flag_pairs_and_when_gate_abstain_cleanly() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [git, commit]
    valueFlags: [-c]
    requireArgs:
      - [-c, core.hooksPath=/dev/null]
  - command: [git, push]
    valueFlags: [-c]
    whenAnyArgs: [origin]
    requireArgs: [HEAD]
"#,
    );
    for script in [
        r#"git -c "$OPT" commit -m hi"#,
        r#"git -c="$OPT" commit -m hi"#,
        r#"git -c"$OPT" commit -m hi"#,
        r#"git push "$REMOTE" main"#,
    ] {
        let hits = scan_script(script, &compiled);
        assert_eq!(
            hits.len(),
            1,
            "expected 1 advisory for `{script}`, got {hits:?}"
        );
        assert!(matches!(hits[0].kind, HitKind::Unanalyzable { .. }));
    }
}
