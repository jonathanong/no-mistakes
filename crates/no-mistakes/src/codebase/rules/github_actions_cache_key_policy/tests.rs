use super::*;

mod coverage;
mod edges;
mod followups;
mod forbidden_paths;
mod policy;

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

fn widget_options() -> options::Options {
    options::Options {
        allow_hash_files: true,
        caches: vec![options::CacheSpec {
            path: ".cache/widgets".into(),
            lockfile: "widgets.lock".into(),
            key_parts: vec!["runner.os".into(), "runner.arch".into()],
        }],
        ..options::Options::default()
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
        caches: vec![
            spec(".cache/widgets", "widgets.lock", &["runner.os"]),
            spec("./.cache/widgets/", "orders.lock", &["runner.os"]),
        ],
        ..options::Options::default()
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

fn spec(path: &str, lockfile: &str, parts: &[&str]) -> options::CacheSpec {
    options::CacheSpec {
        path: path.into(),
        lockfile: lockfile.into(),
        key_parts: parts.iter().map(|part| (*part).to_string()).collect(),
    }
}
