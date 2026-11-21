use super::*;

mod coverage;
mod edges;
mod followups;

fn fixture(name: &str) -> PathBuf {
    crate::test_support::rule_fixture_root(RULE_ID).join(name)
}

fn config(name: &str) -> NoMistakesConfig {
    let path = crate::test_support::rule_fixture_root(RULE_ID)
        .parent()
        .unwrap()
        .join(format!("configs/{name}.yml"));
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn source(root: &str, relative: &str) -> String {
    std::fs::read_to_string(fixture(root).join(relative)).unwrap()
}

fn output_names(run: &str) -> Vec<String> {
    let mut names = Vec::new();
    for event in output::events(run) {
        let output::Event::Write(write) = event else {
            continue;
        };
        if !names.contains(&write.name) {
            names.push(write.name);
        }
    }
    names
}

fn expect_jobs(text: &str, action: bool) -> Vec<scan::Job> {
    match scan::analyze(text, action, &[]) {
        scan::Document::Jobs(jobs) => jobs,
        scan::Document::Invalid { line, detail } => panic!("invalid {line}: {detail}"),
        scan::Document::Unaligned => panic!("unaligned"),
        scan::Document::Ignored => panic!("ignored"),
    }
}

fn rows(root: &str, name: &str) -> Vec<RuleFinding> {
    check(&fixture(root), &config(name)).unwrap()
}

fn file_rows<'a>(findings: &'a [RuleFinding], suffix: &str) -> Vec<&'a RuleFinding> {
    findings
        .iter()
        .filter(|finding| match_file(&finding.file, suffix))
        .collect()
}

fn match_file(file: &str, suffix: &str) -> bool {
    if suffix.contains('/') {
        file.ends_with(suffix)
    } else {
        file.rsplit('/').next() == Some(suffix)
    }
}

fn assert_needles(findings: &[RuleFinding], suffix: &str, needles: &[&str]) {
    let matched = file_rows(findings, suffix);
    assert_eq!(matched.len(), needles.len(), "{suffix}: {matched:?}");
    for needle in needles {
        assert!(
            matched
                .iter()
                .any(|finding| finding.message.contains(needle)),
            "{suffix} missing {needle}: {matched:?}"
        );
    }
}

fn assert_silent(findings: &[RuleFinding], suffix: &str) {
    let matched = file_rows(findings, suffix);
    assert!(matched.is_empty(), "{suffix}: {matched:?}");
}

#[test]
fn fragments_match_whole_expression_tokens() {
    assert!(text::has_fragment("widgets-${{ runner.os }}", "runner.os"));
    assert!(text::has_fragment(
        "widgets-${{  runner.os  }}",
        "runner.os"
    ));
    assert!(!text::has_fragment("widgets-${{ matrix.os }}", "runner.os"));
    assert!(!text::has_fragment(
        "widgets-${{ runner.os_name }}",
        "runner.os"
    ));
    assert!(!text::has_fragment(
        "widgets-${{ runner.os.extra }}",
        "runner.os"
    ));
    assert!(!text::has_fragment(
        "widgets-${{ matrix.runner.os }}",
        "runner.os"
    ));
    assert!(!text::has_fragment(
        "widgets-runner.os-${{ runner.arch }}",
        "runner.os"
    ));
    assert!(!text::has_fragment("${{ 'runner.os' }}", "runner.os"));
    assert!(!text::has_fragment("${{ \"runner.os\" }}", "runner.os"));
    assert!(text::has_fragment(
        "${{ 'runner.os' }}-${{ runner.os }}",
        "runner.os"
    ));
}

#[test]
fn hash_args_require_an_expression_and_keep_each_path() {
    assert_eq!(
        text::hash_args("${{ hashFiles('orders.lock', 'widgets.lock') }}"),
        vec!["orders.lock".to_string(), "widgets.lock".to_string()]
    );
    assert_eq!(
        text::hash_args("${{ hashFiles( 'widgets.lock' ) }}"),
        vec!["widgets.lock".to_string()]
    );
    assert!(text::hash_args("hashFiles('widgets.lock')").is_empty());
}

#[test]
fn output_names_read_the_same_line_as_github_output() {
    let run = "digest=$(sha256sum widgets.lock | awk '{print $1}')\n\
               echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"\n";
    assert_eq!(output_names(run), vec!["digest".to_string()]);
    assert!(text::mentions_path(run, "widgets.lock"));
    assert!(text::mentions_path("cat dir/widgets.lock", "widgets.lock"));
    assert!(!text::mentions_path("echo orders.lock", "widgets.lock"));
    assert!(!text::mentions_path(
        "echo not-widgets.lock",
        "widgets.lock"
    ));
    assert!(output_names("echo \"digest=${digest}\"\n").is_empty());
}

#[test]
fn ordinary_block_steps_keep_key_and_restore_text() {
    let jobs = expect_jobs(&source("policy", ".github/workflows/shared.yml"), false);
    let step = &jobs[0].steps[0];
    assert_eq!(step.key.as_ref().unwrap().value, "widgets-shared");
    assert_eq!(step.restore_keys.as_ref().unwrap().value, "widgets-");
    let flow = expect_jobs(&source("policy", ".github/workflows/flow.yml"), false);
    assert!(flow[0].steps[0].restore_keys.is_some());
    let action = expect_jobs(
        &source("policy", ".github/actions/cache-widgets/action.yml"),
        true,
    );
    assert_eq!(action[0].steps.len(), 1);
}

#[test]
fn anchors_fail_closed_and_non_composite_actions_are_ignored() {
    match scan::analyze(
        &source("policy", ".github/workflows/alias.yml"),
        false,
        &[".cache/widgets".to_string()],
    ) {
        scan::Document::Unaligned => {}
        other => panic!("expected alignment finding, got {other:?}"),
    }
    match scan::analyze(
        &source("policy", ".github/actions/node-action/action.yml"),
        true,
        &[],
    ) {
        scan::Document::Ignored => {}
        other => panic!("expected ignored node action, got {other:?}"),
    }
}

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
    assert_eq!(findings.len(), 46, "{findings:?}");
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
            let proved = judge::judge(relative, &jobs, &widget_options(), &|_| None);
            assert!(proved.is_empty(), "{relative}: {proved:?}");
        }
    }
}

fn widget_options() -> options::Options {
    options::Options {
        reject_restore_keys: true,
        allow_hash_files: true,
        caches: vec![options::CacheSpec {
            path: ".cache/widgets".into(),
            lockfile: "widgets.lock".into(),
            key_parts: vec!["runner.os".into(), "runner.arch".into()],
        }],
    }
}

#[test]
fn hash_files_is_opt_in_and_restore_keys_can_be_allowed() {
    let denied = rows("hash-off", "hash-off");
    assert_needles(&denied, "hash-files.yml", &["allowHashFiles is false"]);
    assert_silent(&denied, "output.yml");
    assert!(rows("restore-ok", "restore-ok").is_empty());
}

#[test]
fn unreadable_workflow_bytes_are_reported() {
    let findings = rows("unreadable", "policy");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".github/workflows/broken.yml");
    assert_eq!(findings[0].line, 2);
    assert!(findings[0].message.contains("UTF-8"));
}

#[test]
fn empty_cache_list_is_a_config_error() {
    let error = check(&fixture("policy"), &config("invalid")).unwrap_err();
    assert!(error
        .to_string()
        .contains("caches must list at least one cache"));
}

#[test]
fn duplicate_paths_and_undotted_fragments_are_rejected() {
    let duplicate = options::Options {
        reject_restore_keys: true,
        allow_hash_files: false,
        caches: vec![
            spec(".cache/widgets", "widgets.lock", &["runner.os"]),
            spec("./.cache/widgets/", "orders.lock", &["runner.os"]),
        ],
    };
    let error = options::validate(&duplicate).unwrap_err().to_string();
    assert!(error.contains("listed more than once"), "{error}");
    let fragment = options::Options {
        caches: vec![spec(".cache/widgets", "widgets.lock", &["os"])],
        ..options::Options::default()
    };
    let error = options::validate(&fragment).unwrap_err().to_string();
    assert!(error.contains("dotted context fragments"), "{error}");
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
    assert!(!super::text::command_hashes_lockfile(
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

fn spec(path: &str, lockfile: &str, parts: &[&str]) -> options::CacheSpec {
    options::CacheSpec {
        path: path.into(),
        lockfile: lockfile.into(),
        key_parts: parts.iter().map(|part| (*part).to_string()).collect(),
    }
}
