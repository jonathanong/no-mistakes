use super::super::*;

fn names(run: &str) -> Vec<String> {
    output::proven_output_names(run, "widgets.lock")
}

#[test]
fn reassignment_drops_a_stale_digest_and_keeps_an_earlier_copy() {
    assert!(names(
        "h=$(sha256sum widgets.lock)\n\
         h=constant\n\
         echo \"digest=$h\" >> \"$GITHUB_OUTPUT\"\n"
    )
    .is_empty());
    assert_eq!(
        names(
            "h=$(sha256sum widgets.lock)\n\
             digest=$h\n\
             h=constant\n\
             echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"\n"
        ),
        vec!["digest".to_string()]
    );
}

#[test]
fn checksum_and_lockfile_must_share_one_command() {
    assert!(text::command_hashes_lockfile(
        "sha256sum widgets.lock | awk '{print $1}'",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "sha256sum widgets.lock && echo done",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "openssl dgst -sha256 widgets.lock",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "sha256sum 'widgets.lock|notes'",
        "widgets.lock"
    ));
    assert!(!text::command_hashes_lockfile(
        "sha256sum orders.lock | grep widgets.lock",
        "widgets.lock"
    ));
    assert!(!text::command_hashes_lockfile(
        "cat widgets.lock | sha256sum",
        "widgets.lock"
    ));
    assert!(!text::command_hashes_lockfile(
        "echo widgets.lock ; sha256sum orders.lock",
        "widgets.lock"
    ));
}

#[test]
fn here_strings_and_arithmetic_shifts_do_not_swallow_output_writes() {
    assert!(output_syntax::heredoc_start("cat <<< \"note\"").is_none());
    assert!(output_syntax::heredoc_start("size=$((1<<3))").is_none());
    assert_eq!(
        output_syntax::heredoc_start("echo $((1<<3)); cat >> \"$GITHUB_OUTPUT\" <<EOF")
            .map(|(delim, _, _)| delim),
        Some("EOF".into())
    );
    assert_eq!(
        names(
            "digest=$(sha256sum widgets.lock)\n\
             cat <<< \"note\"\n\
             size=$((1<<3))\n\
             echo \"digest=$digest\" >> \"$GITHUB_OUTPUT\"\n"
        ),
        vec!["digest".to_string()]
    );
}

#[test]
fn an_output_path_variable_redirects_a_heredoc() {
    assert_eq!(
        names(
            "digest=$(sha256sum widgets.lock)\n\
             OUT=$GITHUB_OUTPUT\n\
             DEST=$OUT\n\
             cat <<EOF >> \"$DEST\"\n\
             digest=${digest}\n\
             EOF\n"
        ),
        vec!["digest".to_string()]
    );
    assert!(names(
        "digest=$(sha256sum widgets.lock)\n\
         OUT=$GITHUB_OUTPUT\n\
         OUT=elsewhere\n\
         echo \"digest=$digest\" >> \"$OUT\"\n"
    )
    .is_empty());
}

#[test]
fn local_action_dirs_reject_parents_and_keep_quotes() {
    assert_eq!(
        local_action::local_action_dir("\"./.github/actions/lock-digest\""),
        Some(".github/actions/lock-digest")
    );
    assert!(local_action::local_action_dir("actions/cache@v4").is_none());
    assert!(local_action::local_action_dir("./../widgets").is_none());
    assert!(local_action::local_action_dir("./").is_none());
    assert!(local_action::local_action_dir("./.github//actions").is_none());
}

#[test]
fn composite_outputs_prove_only_a_declared_run_digest() {
    let workflow = "\
name: Widgets
jobs:
  unit:
    steps:
      - id: lock
        uses: ./.github/actions/lock-digest
      - uses: actions/cache@v4
        with:
          path: .cache/widgets
          key: widgets-${{ runner.os }}-${{ runner.arch }}-${{ steps.lock.outputs.digest }}
";
    let action = "\
name: Lock digest
outputs:
  digest:
    value: ${{ steps.lock.outputs.digest }}
runs:
  using: composite
  steps:
    - id: lock
      run: |
        digest=$(sha256sum widgets.lock)
        echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"
";
    let jobs = match scan::analyze(workflow, false, &[]) {
        scan::Document::Jobs(jobs) => jobs,
        other => panic!("expected jobs, got {other:?}"),
    };
    let options = options::Options {
        reject_restore_keys: true,
        allow_hash_files: false,
        caches: vec![options::CacheSpec {
            path: ".cache/widgets".into(),
            lockfile: "widgets.lock".into(),
            key_parts: vec!["runner.os".into(), "runner.arch".into()],
        }],
    };
    assert!(judge::judge("ci.yml", &jobs, &options, &|uses| {
        (uses == "./.github/actions/lock-digest").then(|| action.to_string())
    })
    .is_empty());
    assert_eq!(judge::judge("ci.yml", &jobs, &options, &|_| None).len(), 1);
    let constant = action.replace(
        "digest=$(sha256sum widgets.lock)\n        echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"",
        "echo \"digest=constant\" >> \"$GITHUB_OUTPUT\"",
    );
    assert_eq!(
        judge::judge("ci.yml", &jobs, &options, &|_| Some(constant.clone())).len(),
        1
    );
    assert_eq!(
        judge::judge("ci.yml", &jobs, &options, &|_| Some(":\n".into())).len(),
        1
    );
    let node = action.replace("using: composite", "using: node20");
    assert_eq!(
        judge::judge("ci.yml", &jobs, &options, &|_| Some(node.clone())).len(),
        1
    );
    let unnamed = action.replace(
        "value: ${{ steps.lock.outputs.digest }}",
        "description: note",
    );
    assert_eq!(
        judge::judge("ci.yml", &jobs, &options, &|_| Some(unnamed.clone())).len(),
        1
    );
}
