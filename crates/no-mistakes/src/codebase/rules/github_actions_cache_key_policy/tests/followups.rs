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
    assert!(!text::command_hashes_lockfile("&&", "widgets.lock"));
    assert!(text::command_hashes_lockfile(
        "sha256sum widgets.lock &&",
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
fn a_self_expansion_keeps_the_proven_digest() {
    for rhs in [
        "\"$digest\"",
        "${digest}",
        "\"${digest%% *}\"",
        "\"${digest%%[[:space:]]*}\"",
        "\"${digest:0:16}\"",
        "\"${digest#*}\"",
        "${digest%%tail}",
        "${digest##tail}",
        "${digest%tail}",
        "${digest:0}",
        "\"${digest#*}\";",
    ] {
        let run = format!(
            "digest=$(sha256sum widgets.lock)\ndigest={rhs}\necho \"digest=$digest\" >> \"$GITHUB_OUTPUT\"\n"
        );
        assert_eq!(names(&run), vec!["digest".to_string()], "{rhs}");
    }
    assert!(names(
        "digest=$(sha256sum widgets.lock)\n\
         digest=\"${digest%% *} extra\"\n\
         kept=$digest\n\
         echo \"digest=$kept\" >> \"$GITHUB_OUTPUT\"\n"
    )
    .is_empty());
    assert_eq!(
        names(
            "digest=$(sha256sum widgets.lock)\n\
             other=\"${digest%% *}\"\n\
             echo \"digest=${digest}\" >> \"$GITHUB_OUTPUT\"\n"
        ),
        vec!["digest".to_string()]
    );
    assert!(names(
        "digest=$(sha256sum widgets.lock)\n\
         other=\"${digest%% *}\"\n\
         echo \"digest=$other\" >> \"$GITHUB_OUTPUT\"\n"
    )
    .is_empty());
    for rhs in [
        "constant",
        "$(cat widgets.lock)",
        "$(sha256sum orders.lock)",
        "\"${other%% *}\"",
        "\"prefix${digest}\"",
        "\"${digest%%}\"",
        "\"${digest:x}\"",
    ] {
        let run = format!(
            "digest=$(sha256sum widgets.lock)\ndigest={rhs}\necho \"digest=$digest\" >> \"$GITHUB_OUTPUT\"\n"
        );
        assert!(names(&run).is_empty(), "{rhs}");
    }
    assert!(names(
        "digest=\"${digest%% *}\"\n\
         echo \"digest=$digest\" >> \"$GITHUB_OUTPUT\"\n"
    )
    .is_empty());
}

#[test]
fn same_name_expansions_match_only_the_whole_value() {
    let keep = [
        "$digest",
        "\"$digest\"",
        "${digest}",
        "\"${digest}\"",
        "${digest%% *}",
        "${digest%%[[:space:]]*}",
        "${digest##*}",
        "${digest%tail}",
        "${digest#*}",
        "${digest:0}",
        "${digest:0:16}",
        "\"${digest#*}\";",
    ];
    for value in keep {
        assert!(expand::same_name_expansion(value, "digest"), "{value}");
    }
    let drop = [
        "$other",
        "$digest extra",
        "${other}",
        "${digest_extra}",
        "${digest%%}",
        "${digest#}",
        "${digest%%$other}",
        "${digest/old/new}",
        "${digest:-fallback}",
        "${digest:}",
        "${digest:0:}",
        "${digest:0:16x}",
        "${digest:x}",
        "prefix${digest}",
        "${digest%% *} extra",
        "constant",
        "",
    ];
    for value in drop {
        assert!(!expand::same_name_expansion(value, "digest"), "{value}");
    }
}

#[test]
fn checksum_pipes_and_redirects_still_name_the_lockfile() {
    assert!(text::command_hashes_lockfile(
        "sha256sum widgets.lock | cut -c1-64",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "shasum -a 256 widgets.lock | awk '{print $1}'",
        "widgets.lock"
    ));
    assert!(text::command_hashes_lockfile(
        "sha256sum < widgets.lock | cut -c1-64",
        "widgets.lock"
    ));
    assert!(output_syntax::heredoc_start("cat <<< \"note\"").is_none());
    assert!(output_syntax::heredoc_start("size=$(( 1 << 2 ))").is_none());
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

#[test]
fn composite_edges_without_a_named_run_are_not_proof() {
    let step = scan::Step {
        line: 1,
        id: Some("lock".into()),
        uses: Some("./.github/actions/lock-digest".into()),
        run: None,
        path: None,
        key: None,
        restore_keys: None,
    };
    let key = "widgets-${{ steps.lock.outputs.digest }}";
    let prove = |source: &str| {
        local_action::proves(&step, key, "widgets.lock", &|_| Some(source.to_string()))
    };
    let numbered = "\
name: Lock digest
outputs:
  ? 7
  :
    value: ${{ steps.lock.outputs.digest }}
runs:
  using: composite
  steps:
    - id: lock
      run: echo \"digest=constant\" >> \"$GITHUB_OUTPUT\"
";
    let parsed = serde_yaml::from_str::<serde_yaml::Value>(numbered).unwrap();
    let output_key = parsed["outputs"]
        .as_mapping()
        .unwrap()
        .keys()
        .next()
        .unwrap();
    assert!(output_key.as_str().is_none(), "{output_key:?}");
    assert!(!prove(numbered));
    assert!(!prove(
        "name: Lock digest\n\
         outputs:\n\
           digest:\n\
             value: ${{ steps.lock.outputs.digest }}\n\
         runs:\n\
           using: composite\n\
           steps:\n\
             - run: echo \"digest=constant\" >> \"$GITHUB_OUTPUT\"\n"
    ));
    assert!(!prove(
        "name: Lock digest\n\
         outputs:\n\
           digest:\n\
             value: ${{ steps.lock.outputs.digest }}\n\
         runs:\n\
           using: composite\n\
           steps:\n\
             - id: lock\n\
               uses: ./widgets\n"
    ));
}
