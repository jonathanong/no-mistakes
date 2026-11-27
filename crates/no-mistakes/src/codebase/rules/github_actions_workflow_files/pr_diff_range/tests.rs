use super::options::CompiledOptions;
use source::check_source_with_options;

mod consistent_pairs;
mod source;

fn check_source(rel: &str, source: &str) -> Vec<no_mistakes_rule_kernel::finding::RuleFinding> {
    check_source_with_options(rel, source, &CompiledOptions::default()).0
}

const BASE: &str = "${{ github.event.pull_request.base.sha }}";
const HEAD: &str = "${{ github.event.pull_request.head.sha }}";

fn workflow(on: &str, run: &str) -> String {
    format!(
        "on: {on}\njobs:\n  changes:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {run}\n"
    )
}

fn hits(on: &str, run: &str) -> usize {
    check_source(".github/workflows/ci.yml", &workflow(on, run)).len()
}

#[test]
fn two_dot_and_two_argument_base_head_ranges_are_reported() {
    assert_eq!(
        hits(
            "pull_request",
            &format!("git diff --name-only {BASE} {HEAD}")
        ),
        1
    );
    assert_eq!(
        hits("pull_request", &format!("git diff '{BASE}..{HEAD}'")),
        1
    );
    assert_eq!(
        hits(
            "[pull_request]",
            &format!("files=$(git diff {BASE} {HEAD})")
        ),
        1
    );
}

#[test]
fn base_paired_with_github_sha_is_reported_when_other_events_can_run() {
    for range in [
        format!("{BASE}...${{{{ github.sha }}}}"),
        format!("{BASE}..${{{{ github.sha }}}}"),
        "origin/${{ github.base_ref }} ${{ github.sha }}".to_string(),
    ] {
        assert_eq!(
            hits("[pull_request, push]", &format!("git diff {range}")),
            1,
            "{range}"
        );
        // A merge_group workflow also reports the empty pull_request read.
        let reads = range.contains("pull_request.");
        assert_eq!(
            hits("[pull_request, merge_group]", &format!("git diff {range}")),
            1 + usize::from(reads),
            "{range}"
        );
        assert_eq!(
            hits("pull_request", &format!("git diff {range}")),
            0,
            "{range}"
        );
        assert_eq!(hits("pull_request_target", &format!("git diff {range}")), 0);
    }
}

#[test]
fn merge_base_forms_are_not_reported() {
    assert_eq!(
        hits("pull_request", &format!("git diff {BASE}...{HEAD}")),
        0
    );
    assert_eq!(hits("pull_request", "git diff HEAD^1 HEAD"), 0);
    assert_eq!(hits("pull_request", "git diff --stat -- docs"), 0);
    assert_eq!(hits("pull_request", &format!("git log {BASE} {HEAD}")), 0);
    assert_eq!(
        hits("pull_request", &format!("git diff -U 3 {BASE} {HEAD}")),
        0
    );
}

#[test]
fn merge_group_ranges_and_other_events_are_not_reported() {
    let range = "${{ github.event.merge_group.base_sha }} ${{ github.event.merge_group.head_sha }}";
    assert_eq!(hits("merge_group", &format!("git diff {range}")), 0);
    assert_eq!(hits("push", &format!("git diff {BASE} {HEAD}")), 0);
}

#[test]
fn merge_group_workflows_reading_pull_request_fields_are_reported() {
    assert_eq!(hits("merge_group", &format!("echo {BASE}")), 1);
    assert_eq!(
        hits("[pull_request, merge_group]", &format!("echo {HEAD}")),
        1
    );
    assert_eq!(hits("pull_request", &format!("echo {BASE}")), 0);
}

#[test]
fn guarded_and_env_resolved_forms_follow_the_declared_values() {
    let guarded = format!(
        "on: [merge_group, pull_request]\njobs:\n  a:\n    runs-on: x\n    steps:\n      - if: github.event_name == 'pull_request'\n        run: echo {BASE}\n"
    );
    assert!(check_source(".github/workflows/ci.yml", &guarded).is_empty());
    let env = format!(
        "on: pull_request\njobs:\n  a:\n    runs-on: x\n    env:\n      B: {BASE}\n      H: {HEAD}\n    steps:\n      - run: git diff \"$B\" ${{H}}\n"
    );
    assert_eq!(check_source(".github/workflows/ci.yml", &env).len(), 1);
}

#[test]
fn composite_actions_and_invalid_yaml_are_ignored() {
    let body = workflow("pull_request", &format!("git diff {BASE} {HEAD}"));
    assert!(check_source(".github/actions/x/action.yml", &body).is_empty());
    assert!(check_source(".github/workflows/ci.yml", "on: [").is_empty());
}

const WF: &str = ".github/workflows/ci.yml";

fn count(yaml: &str) -> usize {
    check_source(WF, yaml).len()
}

#[test]
fn malformed_workflow_shapes_are_skipped() {
    let steps =
        "    steps:\n      - nope\n      - uses: example/action\n      - run: git diff a b\n";
    assert_eq!(count("on: pull_request\njobs:\n  a: nope\n"), 0);
    assert_eq!(count(&format!("on: pull_request\njobs:\n  a:\n{steps}")), 0);
    assert_eq!(count("on: pull_request\n"), 0);
    assert_eq!(count("jobs:\n  a:\n    steps:\n      - run: echo\n"), 0);
    assert_eq!(count("on: 5\njobs: {}\n"), 0);
}

#[test]
fn trigger_forms_select_pull_request_judging() {
    let run = format!("git diff {BASE} {HEAD}");
    for on in [
        "pull_request_target",
        "[push, pull_request]",
        "true: pull_request",
    ] {
        let yaml = if on.starts_with("true") {
            format!("{on}\njobs:\n  a:\n    steps:\n      - run: {run}\n")
        } else {
            format!("on: {on}\njobs:\n  a:\n    steps:\n      - run: {run}\n")
        };
        assert_eq!(count(&yaml), 1, "{on}");
    }
    let mapping = format!("on:\n  pull_request:\njobs:\n  a:\n    steps:\n      - run: {run}\n");
    assert_eq!(count(&mapping), 1);
}

#[test]
fn shell_prefixes_and_git_globals_do_not_hide_a_diff() {
    for command in [
        "git -C . diff",
        "git --no-pager diff",
        "if git diff",
        "FOO=1 git diff",
        "sudo git diff",
        "files=$(git diff",
    ] {
        assert_eq!(
            hits("pull_request", &format!("{command} {BASE} {HEAD}")),
            1,
            "{command}"
        );
    }
    assert_eq!(hits("pull_request", &format!("echo {BASE} {HEAD}")), 0);
    assert_eq!(
        hits("pull_request", &format!("git status {BASE} {HEAD}")),
        0
    );
    assert_eq!(hits("pull_request", "git"), 0);
}

#[test]
fn unrecognized_ranges_are_silent() {
    for range in [
        "${{ github.event.pull_request.base.sha",
        "$A $B",
        "$ $1",
        "a b c",
        "a",
        "--stat",
    ] {
        assert_eq!(
            hits("pull_request", &format!("git diff {range}")),
            0,
            "{range}"
        );
    }
}

#[test]
fn merge_group_reads_follow_env_references() {
    let yaml = |run: &str| {
        format!(
            "on: merge_group\nenv:\n  B: {BASE}\n  N: 1\n  U: {HEAD}\njobs:\n  a:\n    steps:\n      - run: {run}\n"
        )
    };
    assert_eq!(count(&yaml("echo \"$B\"")), 1);
    assert_eq!(count(&yaml("echo ${B}")), 1);
    assert_eq!(count(&yaml("echo $N")), 0);
}

#[test]
fn saved_fixtures_report_only_unsound_ranges() {
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-pr-diff-range/fixture");
    let run = |name: &str, files: &[&str]| {
        let root = crate::codebase::ts_resolver::normalize_path(&base.join(name));
        let files: Vec<_> = files.iter().map(|file| root.join(file)).collect();
        let mut config = crate::config::v2::NoMistakesConfig::default();
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: super::RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            ..Default::default()
        });
        super::check_with_files(&root, &config, &files)
            .unwrap()
            .len()
    };
    let workflow = |file: &str| format!(".github/workflows/{file}");
    let report = [
        workflow("paired.yml"),
        workflow("pull.yml"),
        workflow("queue.yml"),
    ];
    assert_eq!(run("report", &[&report[0], &report[1], &report[2]]), 6);
    let pass = [
        workflow("pass.yml"),
        workflow("pull-only.yml"),
        workflow("push.yml"),
    ];
    assert_eq!(run("pass", &[&pass[0], &pass[1], &pass[2]]), 0);
}

#[test]
fn environment_reads_point_at_the_env_entry() {
    let yaml = format!(
        "on: merge_group\njobs:\n  a:\n    env:\n      B: {BASE}\n    steps:\n      - run: echo \"$B\"\n"
    );
    let found = check_source(WF, &yaml);
    assert_eq!(found.iter().map(|hit| hit.line).collect::<Vec<_>>(), [5]);
}

#[test]
fn shell_guard_anchors_keywords_after_heredoc_bodies() {
    let yaml = |run: &str| {
        format!(
            "on: [pull_request, merge_group]\njobs:\n  a:\n    env:\n      EVENT_NAME: ${{{{ github.event_name }}}}\n      PR_BASE_SHA: {BASE}\n    steps:\n      - run: |\n{run}"
        )
    };
    let only_in_then = concat!(
        "          if [ \"$EVENT_NAME\" = \"pull_request\" ]; then\n",
        "            cat <<EOF\n",
        "          $PR_BASE_SHA\n",
        "          EOF\n",
        "          fi\n",
    );
    assert_eq!(count(&yaml(only_in_then)), 0);
    let after_fi = format!("{only_in_then}          git diff \"$PR_BASE_SHA\" HEAD\n");
    assert_eq!(count(&yaml(&after_fi)), 1);
}

#[test]
fn shell_guard_handles_reassignment_runner_var_and_edge_shapes() {
    let with_env = |run: &str| {
        format!(
            "on: [pull_request, merge_group]\njobs:\n  a:\n    env:\n      EVENT_NAME: ${{{{ github.event_name }}}}\n      PR_BASE_SHA: {BASE}\n      MERGE_GROUP_BASE_SHA: ${{{{ github.event.merge_group.base_sha }}}}\n    steps:\n      - run: |\n{run}"
        )
    };
    // Reassignments before the guard invalidate the event-name comparison.
    for mutation in [
        "          EVENT_NAME=pull_request\n",
        "          export EVENT_NAME=pull_request\n",
        "          declare -x EVENT_NAME=pull_request\n",
        "          unset EVENT_NAME\n",
        "          read -r EVENT_NAME <<< pull_request\n",
        "          printf -v EVENT_NAME pull_request\n",
        "          eval 'EVENT_NAME=pull_request'\n",
        "          source ./env.sh\n",
        "          command export EVENT_NAME=pull_request\n",
        "          command -p -- unset EVENT_NAME\n",
        "          command eval 'EVENT_NAME=pull_request'\n",
        "          builtin declare -x EVENT_NAME=pull_request\n",
        "          command \"$RUNNER_CMD\"\n",
    ] {
        let script = format!(
            "{mutation}          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n"
        );
        assert_eq!(count(&with_env(&script)), 1, "{mutation}");
    }
    // Non-bracket tests, positional variables, unknown operators, glob patterns, unterminated
    // quotes, and unsupported nested control flow in then/else/condition/nested-if all bail out.
    for unguarded in [
        "          if test \"$EVENT_NAME\" = pull_request x; then base=\"$PR_BASE_SHA\"; fi\n",
        "          if [ \"$1\" = pull_request ]; then base=\"$PR_BASE_SHA\"; fi\n",
        "          if [ \"$EVENT_NAME\" -eq pull_request ]; then base=\"$PR_BASE_SHA\"; fi\n",
        "          if [[ \"$EVENT_NAME\" == merge_* ]]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n",
        "          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"\n",
        "          if [ \"$EVENT_NAME\" = pull_request ]; then echo \"$PR_BASE_SHA\" \"unterminated\n",
        "          if [ \"$EVENT_NAME\" = pull_request ]; then (echo \"$PR_BASE_SHA\"); fi\n",
        "          if [ \"$EVENT_NAME\" = pull_request ] || (true); then base=\"$PR_BASE_SHA\"; fi\n",
        "          if [ \"$EVENT_NAME\" = pull_request ]; then if :; then (echo \"$PR_BASE_SHA\"); fi; fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then echo ok; else (echo \"$PR_BASE_SHA\"); fi\n",
        "          for x in 1; do echo \"$x\"; done\n          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; fi\n",
    ] {
        assert_eq!(count(&with_env(unguarded)), 1, "{unguarded}");
    }
    // Multiple guarded ifs, != non-merge_group, nested if/elif/fi, and elif after an already-taken
    // merge_group branch stay quiet.
    let multi = concat!(
        "          set -euo pipefail\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n",
        "            base=\"$MERGE_GROUP_BASE_SHA\"\n",
        "          elif [ \"$EVENT_NAME\" = pull_request ]; then\n",
        "            base=\"$PR_BASE_SHA\"\n",
        "          else\n",
        "            base=\"$PR_BASE_SHA\"\n",
        "          fi\n",
        "          if [ \"$EVENT_NAME\" != pull_request ]; then\n",
        "            echo \"$MERGE_GROUP_BASE_SHA\"\n",
        "          else\n",
        "            if [ -n \"$PR_BASE_SHA\" ]; then\n",
        "              echo \"$PR_BASE_SHA\"\n",
        "            elif :; then\n",
        "              echo \"$PR_BASE_SHA\"\n",
        "            fi\n",
        "          fi\n",
        "          if [ \"$EVENT_NAME\" != merge_group ]; then\n",
        "            echo \"$PR_BASE_SHA\"\n",
        "          fi\n",
    );
    assert_eq!(count(&with_env(multi)), 0);
    // Semicolon-separated `if` without intervening whitespace is still recognized.
    assert_eq!(
        count(&with_env(
            "          set -e;if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n"
        )),
        0
    );
}

#[test]
fn shell_guard_allowlist_and_unmodelled_controls_fail_safe() {
    let with_env = |run: &str| {
        format!(
            "on: [pull_request, merge_group]\njobs:\n  a:\n    env:\n      EVENT_NAME: ${{{{ github.event_name }}}}\n      PR_BASE_SHA: {BASE}\n      MERGE_GROUP_BASE_SHA: ${{{{ github.event.merge_group.base_sha }}}}\n    steps:\n      - run: |\n{run}"
        )
    };
    for mutating in [
        "          $(echo unset) EVENT_NAME\n",
        "          `echo unset` EVENT_NAME\n",
        "          ${CMD:-unset} EVENT_NAME\n",
        "          echo \"${EVENT_NAME:=pull_request}\"\n",
        "          echo ok > \"${EVENT_NAME:=pull_request}\"\n",
        "          : <<< \"$(echo hi)\"\n",
        "          echo $((EVENT_NAME=1))\n",
        "          echo $[EVENT_NAME=1]\n",
        "          (( EVENT_NAME = 1 ))\n",
        "          echo ok\n          (( EVENT_NAME = 1 ))\n",
        "          if [[ EVENT_NAME=1 -eq 1 ]]; then :; fi\n",
        "          [ -v \"a[EVENT_NAME=1]\" ]\n",
        "          test -v \"a[EVENT_NAME=1]\"\n",
        "          [ \"$OP\" \"a[EVENT_NAME=1]\" ]\n",
        "          b=([EVENT_NAME=1]=v)\n",
        "          declare -a b=([EVENT_NAME=1]=v)\n",
        "          echo $'a\\'b' $'c\\'d' \"$(echo hi)\"\n",
        "          echo $'a\\'b' $'c\\'d'\n          (( EVENT_NAME = 1 ))\n",
        "          cat <<EOF\n          it's ok\n          EOF\n          (( EVENT_NAME = 1 ))\n",
        "          (( EVENT_NAME = 1 )) <<EOF\n          hi\n          EOF\n",
        "          command \"-$FLAG\" echo hi\n",
        "          export \"-$FLAG\" OTHER=1\n",
        "          printf -x foo\n",
        "          declare -i n\n          n=\"EVENT_NAME=1\"\n",
        "          V=EVENT_NAME\n          export \"$V=pull_request\"\n",
        "          V=EVENT_NAME\n          unset \"$V\"\n",
        "          declare -n ref=EVENT_NAME\n",
        "          export OTHER=$VAL\n",
        "          printf \"$FLAG\" EVENT_NAME pull_request\n",
        "          printf -v \"$V\" pull_request\n",
        "          ! EVENT_NAME=pull_request\n",
        "          if EVENT_NAME=pull_request; then :; fi\n",
        "          if ! EVENT_NAME=pull_request; then :; fi\n",
        "          if [ \"$EVENT_NAME\" = push ]; then :; elif EVENT_NAME=pull_request; then :; fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then :; fi > \"${EVENT_NAME:=pull_request}\"\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then :; fi\n          (( EVENT_NAME = 1 ))\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            if :; then :; fi > \"${EVENT_NAME:=pull_request}\"\n          fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            if :; then :; fi trailing\n          fi\n",
        "          prepare_env\n",
        "          env EVENT_NAME=pull_request true\n",
    ] {
        let script = format!(
            "{mutating}          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n"
        );
        assert_eq!(count(&with_env(&script)), 1, "{mutating}");
    }
    for unmodelled in [
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            time if false; then :; else base=\"$PR_BASE_SHA\"; fi\n          fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            if time if false; then :; else base=\"$PR_BASE_SHA\"; fi; then :; fi\n          fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            echo ok\n            (( EVENT_NAME = 1 ))\n            base=\"$PR_BASE_SHA\"\n          fi\n",
        "          if [ \"$EVENT_NAME\" = push ]; then :; elif time false; then base=\"$PR_BASE_SHA\"; fi\n",
        "          time if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            coproc if false; then :; else base=\"$PR_BASE_SHA\"; fi\n          fi\n",
    ] {
        assert_eq!(count(&with_env(unmodelled)), 1, "{unmodelled}");
    }
    for safe_preamble in [
        "          printf\n",
        "          printf -- ok\n",
        "          printf '%s\\n' \"$EVENT_NAME\"\n",
        "          export -- OTHER=\"1\"\n",
        "          printf -v OTHER pull_request\n",
        "          echo '${EVENT_NAME:=pull_request}'\n",
        "          echo \"\\$EVENT_NAME\" a\\ b ${EVENT_NAME} # comment\n",
        "          if [[ \"$MERGE_GROUP_BASE_SHA\" =~ ^[0-9a-f]+$ ]]; then :; fi\n",
        "          [ -z foo ]\n",
        "          [ -z \"a[x=5]\" ]\n",
        "          [ -v OTHER ]\n",
        "          echo before\n          cat <<'EOF'\n          (( not arithmetic ))\n          EOF\n          echo after\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            if [ -n \"$MERGE_GROUP_BASE_SHA\" ]; then echo ok; fi\n          fi\n",
    ] {
        let script = format!(
            "{safe_preamble}          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n"
        );
        assert_eq!(count(&with_env(&script)), 0, "{safe_preamble}");
    }
}

#[test]
fn shell_guard_skips_quoted_heredocs_and_scans_unquoted_heredocs() {
    let with_env = |run: &str| {
        format!(
            "on: [pull_request, merge_group]\njobs:\n  a:\n    env:\n      EVENT_NAME: ${{{{ github.event_name }}}}\n      PR_BASE_SHA: {BASE}\n      MERGE_GROUP_BASE_SHA: ${{{{ github.event.merge_group.base_sha }}}}\n    steps:\n      - run: |\n{run}"
        )
    };
    let guard = "          if [ \"$EVENT_NAME\" = pull_request ]; then base=\"$PR_BASE_SHA\"; else base=\"$MERGE_GROUP_BASE_SHA\"; fi\n";
    for safe in [
        "          cat <<'EOF'\n          $(echo unset) EVENT_NAME\n          `echo unset`\n          ${EVENT_NAME:=pull_request}\n          $((EVENT_NAME=1))\n          EOF\n",
        "          cat <<\"EOF\"\n          $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<\\EOF\n          $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<E'O'F\n          $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<-'EOF'\n          \t$(echo unset) EVENT_NAME\n          \tEOF\n",
        "          echo \"café\"\n          cat <<'EOF'\n          $(echo unset)\n          EOF\n",
        "          cat <<EOF\n          hello $EVENT_NAME ${EVENT_NAME} \\$(literal)\n          EOF\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then cat; fi <<'EOF'\n          $(literal)\n          EOF\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then\n            if :; then cat; fi <<'EOF'\n            $(literal)\n          EOF\n          fi\n",
    ] {
        assert_eq!(count(&with_env(&format!("{safe}{guard}"))), 0, "{safe}");
    }
    for mutating in [
        "          cat <<EOF\n          $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<EOF\n          `echo unset` EVENT_NAME\n          EOF\n",
        "          cat <<EOF\n          ${EVENT_NAME:=pull_request}\n          EOF\n",
        "          cat <<EOF\n          $((EVENT_NAME=1))\n          EOF\n",
        "          cat <<EOF\n          don't skip $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<EOF\n          # $(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<EOF\n          \\\\$(echo unset) EVENT_NAME\n          EOF\n",
        "          cat <<'EOF' > \"${EVENT_NAME:=pull_request}\"\n          $(literal)\n          EOF\n",
        "          cat <<'SAFE' <<UNSAFE\n          $(literal)\n          SAFE\n          $(echo unset) EVENT_NAME\n          UNSAFE\n",
        "          if [ \"$EVENT_NAME\" = merge_group ]; then cat; fi <<EOF\n          $(echo unset)\n          EOF\n",
    ] {
        assert_eq!(count(&with_env(&format!("{mutating}{guard}"))), 1, "{mutating}");
    }
}
