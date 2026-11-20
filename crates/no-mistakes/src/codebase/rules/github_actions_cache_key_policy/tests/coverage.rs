use super::super::*;

fn spec(path: &str, lockfile: &str, parts: &[&str]) -> options::CacheSpec {
    options::CacheSpec {
        path: path.into(),
        lockfile: lockfile.into(),
        key_parts: parts.iter().map(|part| (*part).to_string()).collect(),
    }
}

fn policy(parts: &[&str]) -> options::Options {
    options::Options {
        caches: vec![spec(".cache/widgets", "widgets.lock", parts)],
        ..options::Options::default()
    }
}

fn bare_walker() -> walk::Walker {
    walk::Walker {
        action: false,
        in_jobs: false,
        job_open: false,
        jobs_indent: 0,
        job_indent: None,
        in_steps: false,
        steps_indent: 0,
        step_dash: None,
        in_with: false,
        with_indent: 0,
        jobs: Vec::new(),
        scalar: None,
        seq: None,
    }
}

#[test]
fn option_paths_lockfiles_and_fragments_must_be_usable() {
    let path = options::validate(&policy_with("/", "widgets.lock", &["runner.os"]))
        .unwrap_err()
        .to_string();
    assert!(
        path.contains("caches.path must be a non-empty path"),
        "{path}"
    );

    let lockfile = options::validate(&policy_with(".cache/widgets", "./", &["runner.os"]))
        .unwrap_err()
        .to_string();
    assert!(
        lockfile.contains("caches.lockfile must be a non-empty path"),
        "{lockfile}"
    );

    let parts = options::validate(&policy(&[])).unwrap_err().to_string();
    assert!(
        parts.contains("keyParts must list at least one expression fragment"),
        "{parts}"
    );

    for fragment in ["1runner.os", "runner.", "runner.os.bad-name"] {
        let error = options::validate(&policy(&[fragment]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("dotted context fragments"),
            "{fragment}: {error}"
        );
    }
    assert!(options::validate(&policy(&["_runner.os_name.extra"])).is_ok());
}

fn policy_with(path: &str, lockfile: &str, parts: &[&str]) -> options::Options {
    options::Options {
        caches: vec![spec(path, lockfile, parts)],
        ..options::Options::default()
    }
}

#[test]
fn plain_text_and_action_names_drop_quotes_comments_and_wrappers() {
    assert!(text::is_cache_action("\"actions/cache@v4\""));
    assert!(text::is_cache_action("'actions/cache/restore/'"));
    assert!(text::is_cache_action("ACTIONS/CACHE/SAVE"));
    assert!(!text::is_cache_action("actions/checkout@v4"));
    assert!(!text::is_cache_action("actions/cache-extra@v4"));

    assert_eq!(
        text::yaml_plain("\"widgets\\\"lock\""),
        "\"widgets\\\"lock\""
    );
    assert_eq!(text::yaml_plain("\"widgets.lock\" # note"), "widgets.lock");
    assert_eq!(text::yaml_plain("'orders.lock'"), "orders.lock");
    assert_eq!(
        text::yaml_plain("'orders.lock' extra"),
        "'orders.lock' extra"
    );
    assert_eq!(text::yaml_plain("widgets.lock # note"), "widgets.lock");
    assert_eq!(
        text::yaml_plain("widgets.lock\norders.lock"),
        "widgets.lock\norders.lock"
    );
    assert_eq!(text::norm_path(" ./.cache/widgets/ "), ".cache/widgets");
    assert_eq!(text::norm_path("./"), "");
    assert_eq!(
        text::path_entries("\n.cache/widgets\n\n.cache/orders\n"),
        vec![".cache/widgets".to_string(), ".cache/orders".to_string()]
    );
}

#[test]
fn expression_helpers_skip_broken_calls_and_commented_shell() {
    assert!(!text::has_fragment("${{ runner.os", "runner.os"));
    assert!(!text::has_fragment("${{ runner.os }}", "  "));
    assert!(text::has_fragment(
        "${{ xrunner.os runner.os }}",
        "runner.os"
    ));
    assert!(text::has_fragment("${{ runner.os-extra }}", "runner.os"));

    assert!(text::hash_args("${{ hashFiles('widgets.lock'").is_empty());
    assert!(text::hash_args("${{ hashFiles widgets.lock }}").is_empty());
    assert!(text::hash_args("${{ myhashFiles('widgets.lock') }}").is_empty());
    assert_eq!(
        text::hash_args("${{ hashFiles('widgets.lock', '') }}"),
        vec!["widgets.lock".to_string()]
    );
    assert_eq!(
        text::hash_args("${{ hashFiles + hashFiles(\"orders.lock\") }}"),
        vec!["orders.lock".to_string()]
    );
    assert_eq!(
        text::hash_args("${{ hashFiles('widgets.lock') + hashFiles('orders.lock') }}"),
        vec!["widgets.lock".to_string(), "orders.lock".to_string()]
    );

    assert!(!text::mentions_path(
        "# widgets.lock\necho orders.lock",
        "widgets.lock"
    ));
    assert!(text::mentions_path(
        "cat widgets.lock # keep",
        "widgets.lock"
    ));
    assert!(!text::mentions_path("echo widgets.lock", ""));
    assert_eq!(
        text::output_names(
            "echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"\n\
             echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"\n\
             echo 'other=${other}' >> $GITHUB_OUTPUT\n\
             echo _extra=${extra} >> $GITHUB_OUTPUT\n\
             echo 1bad=${bad} >> $GITHUB_OUTPUT\n\
             echo digest >> $GITHUB_OUTPUT\n\
             printf name=1 >> $GITHUB_OUTPUT\n"
        ),
        vec![
            "digest".to_string(),
            "other".to_string(),
            "_extra".to_string()
        ]
    );
}

#[test]
fn flow_scalars_accept_numbers_bools_sequences_and_reject_the_rest() {
    let mut step = scan::Step::blank(4);
    flow::fill_flow(
        &mut step,
        4,
        "{ id: lock, uses: actions/cache, run: echo, with: { path: .cache/widgets, key: 7, restore-keys: [widgets-, orders-] } }",
    );
    assert_eq!(step.id.as_deref(), Some("lock"));
    assert_eq!(step.uses.as_deref(), Some("actions/cache"));
    assert_eq!(step.run.as_deref(), Some("echo"));
    assert_eq!(step.path.as_ref().unwrap().value, ".cache/widgets");
    assert_eq!(step.key.as_ref().unwrap().value, "7");
    assert_eq!(
        step.restore_keys.as_ref().unwrap().value,
        "widgets-\norders-"
    );

    let mut skipped = scan::Step::blank(1);
    flow::fill_flow(&mut skipped, 1, "{");
    flow::fill_flow(&mut skipped, 1, "[1, 2]");
    flow::fill_flow(&mut skipped, 1, "{ uses: actions/cache, with: plain }");
    assert!(skipped.path.is_none());
    assert_eq!(skipped.uses.as_deref(), Some("actions/cache"));

    let mut with = scan::Step::blank(2);
    flow::fill_with_text(&mut with, 2, "{");
    flow::fill_with_text(&mut with, 2, "[]");
    flow::fill_with_text(
        &mut with,
        2,
        "{ path: true, key: [], restore-keys: { nested: 1 } }",
    );
    assert_eq!(with.path.as_ref().unwrap().value, "true");
    assert!(with.key.is_none());
    assert!(with.restore_keys.is_none());
    flow::fill_with_text(&mut with, 2, "{ key: [1, widgets], path: null }");
    assert_eq!(with.key.as_ref().unwrap().value, "widgets");
    assert_eq!(with.path.as_ref().unwrap().value, "true");
}

#[test]
fn yaml_cursors_reject_keys_that_are_not_plain() {
    assert_eq!(
        flow::split_kv("uses: actions/cache"),
        Some(("uses", "actions/cache"))
    );
    assert!(flow::split_kv("no-colon").is_none());
    assert!(flow::split_kv(": empty").is_none());
    assert!(flow::split_kv("bad key: x").is_none());

    assert_eq!(
        flow::list_item("  - uses: actions/cache"),
        Some((2, "uses: actions/cache"))
    );
    assert!(flow::list_item("uses: actions/cache").is_none());
    assert!(flow::list_item("-uses").is_none());

    assert!(flow::mapping_key("  runs/using: composite").is_some());
    assert!(flow::mapping_key("  - uses: actions/cache").is_none());
    assert!(flow::mapping_key("  # comment").is_none());
    assert!(flow::mapping_key("  not a key").is_none());
    assert!(flow::mapping_key("  bad key: x").is_none());

    assert_eq!(flow::comment_value(" widgets # note"), "widgets");
    assert_eq!(
        flow::comment_value(" \"widgets # not\" "),
        "\"widgets # not\""
    );
    assert_eq!(flow::comment_value(" |"), "|");
    assert_eq!(
        flow::comment_value(" { path: .cache/widgets }"),
        "{ path: .cache/widgets }"
    );
    assert_eq!(flow::comment_value(" [widgets]"), "[widgets]");

    assert!(flow::block_header("|"));
    assert!(flow::block_header(">+2"));
    assert!(flow::block_header(">#"));
    assert!(!flow::block_header(""));
    assert!(!flow::block_header("|bad"));
    assert!(!flow::block_header("plain"));
    assert_eq!(flow::leading("   key"), 3);
}

#[test]
fn edits_with_no_open_step_are_ignored() {
    let mut walker = bare_walker();
    assert!(!edit::consume_scalar(&mut walker, "  widgets"));
    edit::finish_scalar(&mut walker);
    edit::assign(&mut walker, edit::Slot::Run, 2, 0, "echo\nwidgets.lock");
    edit::assign(&mut walker, edit::Slot::Key, 2, 0, "\"widgets\"");
    edit::open_with(&mut walker, 2, 0, "inline");
    edit::open_with(&mut walker, 2, 0, "{ path: .cache/widgets }");
    assert!(!walker.in_with);
    edit::open_with(&mut walker, 2, 0, "");
    assert!(walker.in_with);

    walker.seq = Some((edit::Seq::Path, 2));
    assert!(edit::consume_seq(&mut walker, 3, "    - .cache/widgets"));
    assert!(!edit::consume_seq(&mut walker, 4, "  - .cache/orders"));
    assert!(walker.seq.is_none());
    walker.seq = Some((edit::Seq::Restore, 2));
    assert!(!edit::consume_seq(&mut walker, 5, "key: widgets"));
    assert!(walker.seq.is_none());
    assert!(walker.jobs.is_empty());
}

const EDGE_WORKFLOW: &str = r#"---
name: Widgets
jobs:
  build:
    env:
      WIDGETS: 1
    steps:
      - id: lock
        run: |
          # widgets.lock

          cat widgets.lock # keep
          echo "digest=${digest}" >> "$GITHUB_OUTPUT"
      - id: "quoted"
        run: echo orders.lock
      - id: ""
        run: echo widgets.lock
      - id: bare
      - name: note
        env: |
          WIDGETS=1
        with: inline
      - uses: actions/cache/restore@v4
        with: { path: ./.cache/widgets/, key: "  ", restore-keys: "   " }
        timeout-minutes: 5
      - uses: actions/cache@v4
        with:
          path:
            - .cache/widgets
            - ".cache/orders"
          key: widgets-${{ runner.os }}-${{ runner.arch }}
          restore-keys:
            - widgets-
          extra: ignored
          note: |
            keep
      - uses: actions/checkout@v4
      # between
    outputs:
      digest: ${{ steps.lock.outputs.digest }}
  pack:
    steps:
      - uses: actions/checkout@v4
...
"#;

#[test]
fn block_scalars_flow_with_and_blank_restore_keys_keep_their_text() {
    let jobs = match scan::analyze(EDGE_WORKFLOW, false) {
        scan::Document::Jobs(jobs) => jobs,
        other => panic!("expected jobs, got {other:?}"),
    };
    assert_eq!(jobs.len(), 2);
    let steps = &jobs[0].steps;
    assert_eq!(steps.len(), 8, "{steps:?}");
    assert_eq!(
        steps[0].run.as_deref(),
        Some("# widgets.lock\n\ncat widgets.lock # keep\necho \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"")
    );
    assert_eq!(steps[1].id.as_deref(), Some("quoted"));
    assert_eq!(steps[2].id.as_deref(), Some(""));
    assert_eq!(steps[3].id.as_deref(), Some("bare"));
    assert!(steps[3].run.is_none());
    assert!(steps[4].uses.is_none());
    assert_eq!(steps[5].key.as_ref().unwrap().value, "  ");
    assert_eq!(steps[5].restore_keys.as_ref().unwrap().value, "   ");
    assert_eq!(
        steps[6].path.as_ref().unwrap().value,
        ".cache/widgets\n.cache/orders"
    );
    assert_eq!(steps[6].restore_keys.as_ref().unwrap().value, "widgets-");
    assert_eq!(steps[7].uses.as_deref(), Some("actions/checkout@v4"));

    let findings = judge::judge("ci.yml", &jobs, &policy(&["runner.os", "runner.arch"]));
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("has no key")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("does not prove a digest")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("sets restore-keys")),
        "{messages:?}"
    );
    assert_eq!(findings.len(), 3, "{messages:?}");
}

#[test]
fn odd_list_items_outside_a_step_do_not_open_a_step() {
    let jobs = walk::walk_jobs(
        "---\n\nsteps:\njobs:\n  build:\n    - not-a-step\n    steps:\n      name: early\n      - uses: actions/checkout@v4\n        - nested\n      key: sibling\n- stray\n...\n",
        false,
    );
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].steps.len(), 1);
    assert_eq!(
        jobs[0].steps[0].uses.as_deref(),
        Some("actions/checkout@v4")
    );
}
