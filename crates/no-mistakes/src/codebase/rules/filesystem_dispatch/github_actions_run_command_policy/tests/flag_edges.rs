use super::super::judge::{scan_script, HitKind};
use super::compile_yaml;

#[test]
fn dynamic_word_can_be_an_inline_when_pair() {
    for (flag, value_flags) in [("--mode", "--mode"), ("-m", "-m")] {
        let compiled = compile_yaml(&format!(
            r#"
commands:
  - command: [tool, run]
    valueFlags: [{value_flags}]
    whenAnyArgs: [[{flag}, write]]
    requireArgs: [--confirm]
"#,
        ));
        let hits = scan_script(r#"tool run "$ARG""#, &compiled);
        assert!(
            matches!(
                hits.first().map(|hit| &hit.kind),
                Some(HitKind::Unanalyzable { .. })
            ),
            "{flag}: {hits:?}"
        );
    }
}

#[test]
fn value_flag_consumption_hides_only_the_next_flag() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    valueFlags: [--label]
    unlessAnyArgs: [--skip]
    requireArgs: [--strict]
"#,
    );
    let hidden = scan_script(r#"tool run "--label${TAIL}" --skip"#, &compiled);
    assert!(
        matches!(
            hidden.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{hidden:?}"
    );
    assert!(scan_script(r#"tool run "--label${TAIL}" --loose --strict"#, &compiled).is_empty());
    let opaque = scan_script(r#"tool run "$OPT" --loose --strict"#, &compiled);
    assert!(
        matches!(
            opaque.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{opaque:?}"
    );
}

#[test]
fn splitting_after_a_real_terminator_stays_a_missing_flag() {
    let hits = scan_script("gh api -f x=1 -- $ENDPOINT\n", &super::default_policy());
    assert!(
        matches!(
            hits.first().map(|hit| &hit.kind),
            Some(HitKind::MissingAnyRequired { .. })
        ),
        "{hits:?}"
    );
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    valueFlags: [--label]
    requireArgs: [--confirm]
"#,
    );
    let consumed = scan_script("tool run --label -- $EXTRA\n", &compiled);
    assert!(
        matches!(
            consumed.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{consumed:?}"
    );
    let ended = scan_script("tool run --label value -- $EXTRA\n", &compiled);
    assert!(
        matches!(
            ended.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{ended:?}"
    );
}

#[test]
fn expanded_source_text_is_not_a_definite_token_match() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    unlessAnyArgs: ["$REMOTE"]
    requireArgs: [--strict]
"#,
    );
    let hits = scan_script(r#"tool run "$REMOTE""#, &compiled);
    assert!(
        matches!(
            hits.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{hits:?}"
    );
}

#[test]
fn dynamic_option_name_is_uncertain_until_the_policy_holds() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    whenAnyArgs: [--write]
    requireArgs: [--strict]
"#,
    );
    let missing = scan_script(r#"tool run "--${MODE}""#, &compiled);
    assert!(
        matches!(
            missing.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{missing:?}"
    );
    let satisfied = scan_script(r#"tool run --strict "--${MODE}""#, &compiled);
    assert!(satisfied.is_empty(), "{satisfied:?}");
    let absent = scan_script("tool run --read", &compiled);
    assert!(absent.is_empty(), "{absent:?}");
    let definite = scan_script("tool run --write", &compiled);
    assert!(
        matches!(
            definite.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{definite:?}"
    );
}

#[test]
fn dynamic_option_can_hide_a_positional_exemption() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [gh, api]
    valueFlags: [--jq]
    unlessAnyArgs: [graphql]
    whenAnyArgs: [-f]
    requireArgs: [--method]
"#,
    );
    let uncertain = scan_script(
        r#"gh api "--${FLAG}" graphql -f x=1 repos/my-org/my-repo/labels"#,
        &compiled,
    );
    assert!(
        matches!(
            uncertain.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{uncertain:?}"
    );
    let literal = scan_script(
        "gh api --jq graphql -f x=1 repos/my-org/my-repo/labels",
        &compiled,
    );
    assert!(
        matches!(
            literal.first().map(|hit| &hit.kind),
            Some(HitKind::MissingRequired { .. })
        ),
        "{literal:?}"
    );
}

#[test]
fn dynamic_option_name_can_match_a_when_pair() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [tool, run]
    whenAnyArgs: [[--mode, write]]
    requireArgs: [--confirm]
"#,
    );
    let separate = scan_script(r#"tool run "--${MODE}" write"#, &compiled);
    assert!(
        matches!(
            separate.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{separate:?}"
    );
    let inline = scan_script(r#"tool run "--${MODE}" read"#, &compiled);
    assert!(
        matches!(
            inline.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{inline:?}"
    );
    let literal = scan_script("tool run --other read", &compiled);
    assert!(literal.is_empty(), "{literal:?}");
}

#[test]
fn escaped_and_nested_substitutions_keep_the_following_command() {
    let compiled = compile_yaml(
        r#"
commands:
  - command: [gh, api]
    valueFlags: [-f, --method]
    whenAnyArgs: [-f]
    requireArgs: [--method]
"#,
    );
    for script in [
        r#"value=$(printf \"); gh api -f x=1 repos/my-org/my-repo/labels"#,
        r#"value=$(printf '%s' "$(printf '"')"); gh api -f x=1 repos/my-org/my-repo/labels"#,
    ] {
        let hits = scan_script(script, &compiled);
        assert!(
            matches!(
                hits.first().map(|hit| &hit.kind),
                Some(HitKind::MissingRequired { .. })
            ),
            "{script}: {hits:?}"
        );
    }
    let escaped_value = scan_script(
        r#"gh api -f x=1 --method="$( : \) )" repos/my-org/my-repo/labels"#,
        &compiled,
    );
    assert!(
        matches!(
            escaped_value.first().map(|hit| &hit.kind),
            Some(HitKind::Unanalyzable { .. })
        ),
        "{escaped_value:?}"
    );
}
