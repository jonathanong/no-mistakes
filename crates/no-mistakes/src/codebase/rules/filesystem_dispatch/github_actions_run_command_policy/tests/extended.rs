use super::super::judge::{scan_script, HitKind};
use super::super::options::{self, Options};
use super::super::runs::check_shell_source;
use super::super::{diagnostics_with_files_and_sources, RULE_ID};
use crate::codebase::ts_source::discover_files;
use crate::config::v2::{load_v2_config, NoMistakesConfig, Unanalyzable};
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
gh api --method "$METHOD" --method GET -f x=1 repos/my-org/my-repo/labels
"#,
        &compiled,
    );
    assert!(ok.is_empty(), "expected no hits, got {ok:?}");

    let definite = scan_script(
        r#"
gh api -f x=1 "repos/${OWNER}/${REPO}/labels"
gh api -f x=1 "${OWNER}/labels"
gh api -f x=1 "g${A}zz${B}l"
gh api -f x=1 -q graphql repos/my-org/my-repo/labels
gh api "graphql?foo=bar" -f query='query { viewer { login } }'
gh api -f x=1 "$ " "$(echo x)"
"#,
        &compiled,
    );
    assert_eq!(definite.len(), 7, "{definite:?}");
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
    assert!(matches!(
        definite[4].kind,
        HitKind::MissingAnyRequired { .. }
    ));
    assert!(matches!(definite[5].kind, HitKind::ForbiddenArg { .. }));
    assert!(matches!(definite[6].kind, HitKind::Unanalyzable { .. }));

    for dynamic in [
        r#"gh api -f x=1 "$ENDPOINT""#,
        r#"gh api -f x=1 "${ENDPOINT:-${ALT}}""#,
        r#"gh api -f x=1 "graphql${SUFFIX}""#,
        r#"gh api -f x=1 "${PREFIX}graphql""#,
        r#"gh api -f x=1 "g${MID}l""#,
        r#"gh api -f x=1 "g${A}ap${B}l""#,
        r#"gh api -f x=1 "$$" "$(echo x)""#,
        r#"gh api -f x=1 "$(get_endpoint)""#,
        r#"gh api -f x=1 `get_endpoint`"#,
        r#"gh api -f x=1 "${{ inputs.endpoint }}""#,
        r#"gh api -f x=1 $EXTRA"#,
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
gh api --method="P${REST}" -f x=1 repos/my-org/my-repo/labels
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
    let err = load_v2_config(&root, Some(&cfg_path)).unwrap_err();
    assert!(format!("{err:#}").contains("options.shellFiles"), "{err:#}");

    let policy_cfg = base.join("configs/policy-shell.yml");
    let mut valid_config = load_v2_config(&root, Some(&policy_cfg)).unwrap();
    let mut unknown_config = valid_config.clone();
    unknown_config.rules[0].options["unknownField"] = serde_yaml::Value::Bool(true);
    let err = crate::codebase::rules::validate_options::validate_rule_options(
        &root,
        &policy_cfg,
        &unknown_config,
        &|_| None,
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("unknownField"), "{err:#}");
    let mut files = discover_files(&root, &valid_config.filesystem.skip_directories);
    files.push(root.join("scripts/missing.sh"));
    let sources = crate::codebase::rules::source_store_for_files(&files);
    valid_config.rules[0].options["shellFiles"] = serde_yaml::Value::Sequence(vec![
        serde_yaml::Value::String("scripts/**/*.sh".into()),
        serde_yaml::Value::String(".github/workflows/*.yml".into()),
    ]);
    let diag =
        diagnostics_with_files_and_sources(&root, &valid_config, &files, &sources, false).unwrap();
    assert!(!diag.findings.is_empty());
    assert!(!diag.advisories.is_empty());
}

#[test]
fn covers_multi_token_matchers_flag_pair_mismatches_and_shell_source_policies() {
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
    requireArgs: [HEAD, [foo, bar]]
  - command: [tool, run]
    requireArgs: [--strict, [-o, a, b]]
"#,
    );
    let ok = scan_script(
        r#"
git push origin HEAD foo bar
tool run --strict -o a b
"#,
        &compiled,
    );
    assert!(ok.is_empty(), "{ok:?}");

    let bad = scan_script(
        r#"
git -c other commit -m hi
git -c=other commit -m hi
git -cother commit -m hi
git -c="other/$X" commit -m hi
git -c"other/$X" commit -m hi
git commit -c
tool run -o a c
git push origin HEAD foo baz
"#,
        &compiled,
    );
    assert!(bad.len() >= 8, "{bad:?}");
    assert!(bad
        .iter()
        .all(|hit| matches!(hit.kind, HitKind::MissingRequired { .. })));

    for script in [
        r#"git -c "$OPT" commit -m hi"#,
        r#"git -c="$OPT" commit -m hi"#,
        r#"git -c"$OPT" commit -m hi"#,
        r#"git push "$REMOTE" main"#,
        r#"git push origin $EXTRA"#,
    ] {
        let hits = scan_script(script, &compiled);
        assert_eq!(
            hits.len(),
            1,
            "expected 1 advisory for `{script}`, got {hits:?}"
        );
        assert!(matches!(hits[0].kind, HitKind::Unanalyzable { .. }));
    }

    let report_diag = check_shell_source(
        "scripts/run.sh",
        "\u{feff}git -c \"$OPT\" commit -m hi\ntool run\n",
        &compiled,
        Unanalyzable::Report,
        None,
    );
    assert_eq!(report_diag.findings.len(), 3, "{:?}", report_diag.findings);
    assert!(report_diag.advisories.is_empty());

    let ignore_diag = check_shell_source(
        "scripts/run.sh",
        "git -c \"$OPT\" commit -m hi\n",
        &compiled,
        Unanalyzable::Ignore,
        None,
    );
    assert!(ignore_diag.findings.is_empty() && ignore_diag.advisories.is_empty());

    let mut cfg = NoMistakesConfig::default();
    cfg.rules.clear();
    crate::codebase::rules::validate_options::validate_rule_options(
        std::path::Path::new("."),
        std::path::Path::new(".no-mistakes.yml"),
        &cfg,
        &|_| None,
    )
    .unwrap();
}

#[test]
fn multi_token_dynamic_words_abstain_instead_of_passing_silently() {
    let when_gate = compile_yaml(
        r#"
commands:
  - command: [git, push]
    whenAnyArgs: [[origin, main]]
    requireArgs: [--force-with-lease]
"#,
    );
    let dynamic_when = scan_script(r#"git push "$REMOTE" main"#, &when_gate);
    assert_eq!(dynamic_when.len(), 1, "{dynamic_when:?}");
    assert!(matches!(dynamic_when[0].kind, HitKind::Unanalyzable { .. }));
    assert!(scan_script("git push upstream feature", &when_gate).is_empty());
    let literal_when = scan_script("git push origin main", &when_gate);
    assert!(matches!(
        literal_when.first().map(|hit| &hit.kind),
        Some(HitKind::MissingRequired { .. })
    ));

    let unless_gate = compile_yaml(
        r#"
commands:
  - command: [git, push]
    unlessAnyArgs: [[origin, main]]
    requireArgs: [--force-with-lease]
"#,
    );
    let dynamic_unless = scan_script(r#"git push "$REMOTE" main"#, &unless_gate);
    assert_eq!(dynamic_unless.len(), 1, "{dynamic_unless:?}");
    assert!(matches!(
        dynamic_unless[0].kind,
        HitKind::Unanalyzable { .. }
    ));
    assert!(scan_script("git push origin main", &unless_gate).is_empty());
    let definite = scan_script("git push upstream feature", &unless_gate);
    assert!(matches!(
        definite.first().map(|hit| &hit.kind),
        Some(HitKind::MissingRequired { .. })
    ));

    let required = compile_yaml(
        r#"
commands:
  - command: [git, push]
    requireArgs: [[origin, main]]
  - command: [tool, run]
    requireArgs: [[--opt, alpha, beta]]
"#,
    );
    assert!(scan_script("git push origin main", &required).is_empty());
    let dynamic_required = scan_script(r#"git push "$REMOTE" main"#, &required);
    assert_eq!(dynamic_required.len(), 1, "{dynamic_required:?}");
    assert!(matches!(
        dynamic_required[0].kind,
        HitKind::Unanalyzable { .. }
    ));
    let missing = scan_script("git push upstream feature", &required);
    assert!(matches!(
        missing.first().map(|hit| &hit.kind),
        Some(HitKind::MissingRequired { .. })
    ));
    assert!(scan_script("tool run --opt alpha beta", &required).is_empty());
    let dynamic_flag = scan_script(r#"tool run --opt "$NAME" beta"#, &required);
    assert_eq!(dynamic_flag.len(), 1, "{dynamic_flag:?}");
    assert!(matches!(dynamic_flag[0].kind, HitKind::Unanalyzable { .. }));
    let missing_flag = scan_script("tool run --opt other beta", &required);
    assert!(matches!(
        missing_flag.first().map(|hit| &hit.kind),
        Some(HitKind::MissingRequired { .. })
    ));
}

#[test]
fn value_flag_consumption_keeps_later_flags_and_rejects_positional_digits() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    valueFlags: [--label]
    whenAnyArgs: [--write]
    requireArgs: [--confirm]
  - command: [tool, emit]
    valueFlags: [--label]
    requireArgs: [[-o, a, b]]
  - command: [gh, api]
    valueFlags: [-f, -F, --field, --raw-field, -X, --method]
    unlessAnyArgs: [graphql]
    requireAnyArgs: [--method, -X]
"#,
    );
    let consumed_terminator = scan_script(r#"tool run --label "--" --write"#, &compiled);
    assert!(
        matches!(
            consumed_terminator.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{consumed_terminator:?}"
    );
    assert!(scan_script(r#"tool run -- --write"#, &compiled).is_empty());
    let consumed_flag = scan_script(r#"tool emit --label "-o" a b"#, &compiled);
    assert!(
        matches!(
            consumed_flag.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{consumed_flag:?}"
    );
    assert!(scan_script("tool emit -o a b", &compiled).is_empty());

    let positional_digit = scan_script(r#"gh api -f x=1 "$1zz""#, &compiled);
    assert!(
        matches!(
            positional_digit.first().map(|hit| &hit.kind),
            Some(HitKind::MissingAnyRequired { .. })
        ),
        "{positional_digit:?}"
    );
    let open_positional = scan_script(r#"gh api -f x=1 "$1""#, &compiled);
    assert!(
        matches!(
            open_positional.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{open_positional:?}"
    );
}

#[test]
fn dynamic_words_do_not_settle_flag_gates_or_consume_exemptions() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    valueFlags: [--label]
    whenAnyArgs: [--write]
    requireArgs: [--confirm]
  - command: [tool, emit]
    valueFlags: [--label]
    unlessAnyArgs: [exempt]
    requireArgs: [--strict]
"#,
    );
    let dynamic_when = scan_script(r#"tool run "$MODE""#, &compiled);
    assert!(
        matches!(
            dynamic_when.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{dynamic_when:?}"
    );
    assert!(
        scan_script("tool run resource", &compiled).is_empty(),
        "a literal non-flag does not activate whenAnyArgs"
    );
    assert!(scan_script("tool run --write --confirm", &compiled).is_empty());
    let missing_confirm = scan_script("tool run --write", &compiled);
    assert!(
        matches!(
            missing_confirm.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{missing_confirm:?}"
    );

    let dynamic_exempt = scan_script(r#"tool emit "$OPT" exempt"#, &compiled);
    assert!(
        matches!(
            dynamic_exempt.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{dynamic_exempt:?}"
    );
    assert!(scan_script("tool emit exempt", &compiled).is_empty());
    let consumed_exempt = scan_script("tool emit --label exempt resource", &compiled);
    assert!(
        matches!(
            consumed_exempt.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{consumed_exempt:?}"
    );
}
