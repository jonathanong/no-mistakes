use super::super::*;
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use super::{assert_needles, assert_silent, expect_jobs, rows, source, widget_options};

#[test]
fn policy_fixture_reports_only_the_configured_cache_gaps() {
    let findings = rows("policy", "policy");
    let platform = "missing platform fragments";
    let digest = "does not prove a digest of `widgets.lock`";
    let restore = "sets restore-keys";
    assert_needles(&findings, "shared.yml", &[platform, digest, restore]);
    assert_needles(
        &findings,
        "matrix.yml",
        &["missing platform fragments `runner.os`, `runner.arch`"],
    );
    assert_needles(
        &findings,
        "os-name.yml",
        &["missing platform fragments `runner.os`."],
    );
    for file in [
        "later-output.yml",
        "wrong-output.yml",
        "no-mention.yml",
        "cross-job.yml",
    ] {
        assert_needles(&findings, file, &[digest]);
    }
    assert_needles(
        &findings,
        "restore-action.yml",
        &[platform, digest, restore],
    );
    assert_needles(&findings, "save-action.yml", &[platform, digest]);
    assert_needles(&findings, "flow.yml", &[platform, digest, restore]);
    assert_needles(&findings, "missing-key.yml", &["has no key", restore]);
    assert_needles(&findings, "sequence-restore.yml", &[restore]);
    assert_needles(&findings, "multiline-path.yml", &[platform, digest]);
    assert_needles(&findings, "alias.yml", &["could not be aligned"]);
    assert_needles(&findings, "aliased-path.yml", &["could not be aligned"]);
    assert_needles(
        &findings,
        "literal-fragment.yml",
        &["missing platform fragments `runner.os`"],
    );
    assert_needles(&findings, "unrelated-output.yml", &[digest]);
    for file in [
        "stale-assign.yml",
        "loose-hash.yml",
        "expand-mixed.yml",
        "mention-output.yml",
        "ampersand-hash.yml",
        "literal-output.yml",
        "split-ampersand.yml",
        "numeric-redirect.yml",
        "suffix-output.yml",
        "glued-suffix.yml",
        "escaped-output.yml",
        "empty-literal.yml",
        "joined-output.yml",
        "quoted-name.yml",
        "composite-plain.yml",
        "composite-missing.yml",
        "composite-unlisted.yml",
    ] {
        assert_needles(&findings, file, &[digest]);
    }
    assert_needles(&findings, "workflows/invalid.yml", &["Invalid YAML"]);
    assert_needles(
        &findings,
        "cache-widgets/action.yml",
        &[platform, digest, restore],
    );
    for file in [
        "hash-files.yml",
        "quoted-hash.yml",
        "spaced-hash.yml",
        "multi-hash.yml",
        "output.yml",
        "other-path.yml",
        "prefix.yml",
        "dot-slash.yml",
        "checkout.yml",
        "suppressed.yml",
        "file-disabled.yml",
        "nested/skip.yml",
        "notes/cache.yml",
        "node-action/action.yml",
        "printf-output.yml",
        "heredoc-output.yml",
        "brace-output.yml",
        "multiline-key.yml",
        "quoted-key.yml",
        "alias-other.yml",
        "aliased-other-path.yml",
        "hash-widgets/action.yml",
        "here-string.yml",
        "shift-output.yml",
        "indirect-output.yml",
        "quoted-output.yml",
        "stderr-merge.yml",
        "both-redirect.yml",
        "tight-redirect.yml",
        "adjacent-redirect.yml",
        "wide-redirect.yml",
        "named-alias.yml",
        "bare-output.yml",
        "empty-expand.yml",
        "quoted-arg.yml",
        "composite-digest.yml",
        "composite-yaml.yml",
        "lock-digest/action.yml",
        "yaml-digest/action.yaml",
        "trim-suffix.yml",
        "expand-quoted.yml",
        "expand-braced.yml",
        "trim-space-class.yml",
        "expand-slice.yml",
        "trim-prefix.yml",
    ] {
        assert_silent(&findings, file);
    }
    assert_scanned_fixtures(&findings);
    assert_eq!(findings.len(), 60, "{findings:?}");
}

fn assert_scanned_fixtures(findings: &[RuleFinding]) {
    for (suffix, relative) in [
        ("printf-output.yml", ".github/workflows/printf-output.yml"),
        ("heredoc-output.yml", ".github/workflows/heredoc-output.yml"),
        ("brace-output.yml", ".github/workflows/brace-output.yml"),
        ("multiline-key.yml", ".github/workflows/multiline-key.yml"),
        ("quoted-key.yml", ".github/workflows/quoted-key.yml"),
        ("alias-other.yml", ".github/workflows/alias-other.yml"),
        (
            "hash-widgets/action.yml",
            ".github/actions/hash-widgets/action.yml",
        ),
        ("here-string.yml", ".github/workflows/here-string.yml"),
        ("shift-output.yml", ".github/workflows/shift-output.yml"),
        (
            "indirect-output.yml",
            ".github/workflows/indirect-output.yml",
        ),
        (
            "composite-digest.yml",
            ".github/workflows/composite-digest.yml",
        ),
        ("composite-yaml.yml", ".github/workflows/composite-yaml.yml"),
        (
            "lock-digest/action.yml",
            ".github/actions/lock-digest/action.yml",
        ),
        (
            "yaml-digest/action.yaml",
            ".github/actions/yaml-digest/action.yaml",
        ),
    ] {
        assert_silent(findings, suffix);
        assert!(gha_kind(relative).is_some(), "{relative}");
        let jobs = expect_jobs(
            &source("policy", relative),
            gha_kind(relative) == Some(true),
        );
        assert!(
            jobs.iter()
                .flat_map(|job| &job.steps)
                .any(|step| step.run.is_some() || step.uses.is_some()),
            "{relative} was not scanned"
        );
        if matches!(
            suffix,
            "printf-output.yml"
                | "heredoc-output.yml"
                | "brace-output.yml"
                | "multiline-key.yml"
                | "quoted-key.yml"
                | "here-string.yml"
                | "shift-output.yml"
                | "indirect-output.yml"
        ) {
            let proved = super::judge_findings(relative, &jobs, &widget_options(), None, &|_| None);
            assert!(proved.is_empty(), "{relative}: {proved:?}");
        }
    }
}

#[test]
fn redirect_words_keep_suffixes_escapes_and_the_next_operator() {
    let targets = shell_cmd::redirect_targets;
    assert_eq!(
        targets("echo é >'x'>$GITHUB_OUTPUT"),
        vec!["x".to_string(), "$GITHUB_OUTPUT".to_string()]
    );
    assert_eq!(
        targets("echo x > \"$OUT\".bak"),
        vec!["$OUT.bak".to_string()]
    );
    assert_eq!(
        targets("echo x >/tmp/orders>$GITHUB_OUTPUT"),
        vec!["/tmp/orders".to_string(), "$GITHUB_OUTPUT".to_string()]
    );
    assert_eq!(
        targets("echo x >/tmp/orders\\>name"),
        vec!["/tmp/orders>name".to_string()]
    );
    assert_eq!(
        targets("echo x >/tmp/orders\\"),
        vec!["/tmp/orders".to_string()]
    );
    assert!(targets("echo x >\"$\"GITHUB_OUTPUT").is_empty());
    assert_eq!(
        targets("echo x >out\" widgets.lock\""),
        vec!["out widgets.lock".to_string()]
    );
    assert!(!text::command_hashes_lockfile(
        "sha256sum orders.lock >out\" widgets.lock\"",
        "widgets.lock"
    ));
    assert_eq!(
        targets("echo x > \"$OUT\"bak"),
        vec!["$OUT.bak".to_string()]
    );
    assert_eq!(
        targets("echo x >\\$GITHUB_OUTPUT"),
        vec!["\\$GITHUB_OUTPUT".to_string()]
    );
    assert!(targets("echo x > '$OUT'\"\"").is_empty());
    assert!(targets("echo x > \"\"'$OUT'").is_empty());
    assert_eq!(targets("echo x > \"$OUT\"''"), vec!["$OUT".to_string()]);
    assert_eq!(targets("echo x > ''\"$OUT\""), vec!["$OUT".to_string()]);
    let prove = |run: &str| output::proven_output_names(run, "widgets.lock");
    assert!(prove(
        "digest=$(sha256sum widgets.lock)\n\
         OUT=$GITHUB_OUTPUT\n\
         OUTbak=$GITHUB_OUTPUT\n\
         echo \"digest=$digest\" > \"$OUT\".bak\n\
         echo \"digest=$digest\" > \"$OUT\"bak\n\
         echo \"digest=$digest\" >\\$GITHUB_OUTPUT\n\
         echo \"digest=$digest\" > '$OUT'\"\"\n\
         echo \"digest=$digest\" > \"\"'$OUT'\n"
    )
    .is_empty());
    assert_eq!(
        prove(
            "digest=$(sha256sum widgets.lock)\n\
             OUTbak=$GITHUB_OUTPUT\n\
             OUT=$GITHUB_OUTPUT\n\
             echo \"digest=$digest\" > \"$OUTbak\"\n\
             echo \"digest=$digest\" >$GITHUB_OUTPUT\n\
             echo \"digest=$digest\" > \"$OUT\"''\n\
             echo \"digest=$digest\" > ''\"$OUT\"\n"
        ),
        vec!["digest".to_string()]
    );
    assert_eq!(
        prove(
            "digest=$(sha256sum widgets.lock)\n\
             echo \"digest=$digest\" >/tmp/orders>\"$GITHUB_OUTPUT\"\n\
             echo \"digest=$digest\" é >'x'>\"$GITHUB_OUTPUT\"\n"
        ),
        vec!["digest".to_string()]
    );
}
