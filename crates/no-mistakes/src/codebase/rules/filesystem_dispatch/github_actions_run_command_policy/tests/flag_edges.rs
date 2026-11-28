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
