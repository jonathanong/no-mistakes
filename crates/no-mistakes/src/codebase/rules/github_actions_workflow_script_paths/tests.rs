use super::options::{self, Options};
use super::scripts::{scan_script, ScriptHitKind, WorkingDir};
use super::triggers::parse_triggers;
use serde_yaml::Value;

fn compile_yaml(yaml: &str) -> anyhow::Result<options::CompiledOptions> {
    let raw: Options = serde_yaml::from_str(yaml)?;
    options::compile(raw)
}

#[test]
fn rejects_invalid_options_at_config_load() {
    for bad in [
        "scriptDirs: ['']",
        "scriptDirs: ['/abs/scripts']",
        "scriptDirs: ['scripts\\sub']",
        "scriptDirs: ['scripts/*']",
        "scriptDirs: ['../outside']",
        "scriptDirs: ['.']",
        "scriptDirs: ['scripts', './scripts/']",
        "include: ['']",
        "include: ['[invalid']",
        "events: ['workflow_dispatch']",
        "events: ['push', 'push']",
        "unknownKey: true",
    ] {
        assert!(compile_yaml(bad).is_err(), "expected error for `{bad}`");
    }
}

#[test]
fn normalizes_valid_script_dirs_and_events() {
    let compiled = compile_yaml("scriptDirs: ['./scripts/', 'tools/ci']\nevents: [pull_request]\n")
        .expect("valid options");
    assert_eq!(compiled.script_dirs, vec!["scripts", "tools/ci"]);
    assert_eq!(compiled.events, vec!["pull_request"]);
}

#[test]
fn triggers_evaluate_paths_negations_and_paths_ignore() {
    let yaml = r#"
on:
  pull_request:
    paths:
      - 'scripts/**'
      - '!scripts/ignored/**'
      - 'scripts/ignored/keep.sh'
  push:
    paths-ignore:
      - 'scripts/**'
"#;
    let value: Value = serde_yaml::from_str(yaml).unwrap();
    let events = vec!["push".to_string(), "pull_request".to_string()];
    let parsed = parse_triggers(yaml, &value, &events);
    assert!(parsed.unanalyzable.is_empty());
    assert_eq!(parsed.filters.len(), 2);

    let pr = parsed
        .filters
        .iter()
        .find(|f| f.event == "pull_request")
        .unwrap();
    assert!(pr.covers("scripts/check.sh"));
    assert!(pr.covers("scripts/sub/check.sh"));
    assert!(!pr.covers("scripts/ignored/skip.sh"));
    assert!(pr.covers("scripts/ignored/keep.sh"));

    let push = parsed.filters.iter().find(|f| f.event == "push").unwrap();
    assert!(!push.covers("scripts/check.sh"));
    assert!(push.covers("apps/web/index.ts"));
}

#[test]
fn triggers_report_unanalyzable_shapes() {
    for bad in [
        "- not-a-mapping",
        "base: &base {}\n<<: *base\njobs: {}",
        "on: 42\njobs: {}",
        "on: [push, 12]\njobs: {}",
        "base: &base {}\non:\n  <<: *base\njobs: {}",
        "on:\n  pull_request: 42\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/**']\n    paths-ignore: ['docs/**']\njobs: {}",
        "on:\n  pull_request:\n    paths: 'scripts/**'\njobs: {}",
        "on:\n  pull_request:\n    paths: ['${{ inputs.glob }}']\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/+(a|b).sh']\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/check.jsx?']\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/check.sh+']\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/pre**post.sh']\njobs: {}",
        "on:\n  pull_request:\n    paths: [42]\njobs: {}",
        "on:\n  pull_request:\n    paths: ['']\njobs: {}",
        "on:\n  pull_request:\n    paths: ['[invalid']\njobs: {}",
        "base: &base {}\non:\n  pull_request:\n    <<: *base\njobs: {}",
    ] {
        let value: Value = serde_yaml::from_str(bad).unwrap();
        let events = vec!["push".to_string(), "pull_request".to_string()];
        let parsed = parse_triggers(bad, &value, &events);
        assert!(
            !parsed.unanalyzable.is_empty(),
            "expected unanalyzable trigger for:\n{bad}"
        );
    }
    for ok in [
        "name: No On\njobs: {}",
        "on: push\njobs: {}",
        "  on: push\n",
        "on:\n  pull_request:\n  # comment\n  push:\n    paths:\n      - '**foo.sh'\njobs: {}",
        "on:\n  pull_request:\n    paths: ['scripts/check.jsx[?]']\njobs: {}",
    ] {
        let value: Value = serde_yaml::from_str(ok).unwrap();
        let events = vec!["push".to_string(), "pull_request".to_string()];
        let parsed = parse_triggers(ok, &value, &events);
        assert!(
            parsed.unanalyzable.is_empty(),
            "unexpected issue for:\n{ok}"
        );
    }
}

#[test]
fn scans_direct_interpreter_runner_cd_and_nested_scripts() {
    let dirs = vec!["scripts".to_string(), "tools/ci".to_string()];
    let script = r#"
./scripts/build.sh
bash -euo pipefail scripts/lint.sh
node --experimental-strip-types scripts/check.mts --strict
pnpm exec tsx tools/ci/verify.ts
bash -c 'python3 scripts/nested.py'
echo "$(sh scripts/sub.sh)"
cd scripts && ./after-cd.sh
echo scripts/not-executed.sh
node -e 'console.log("scripts/inline.sh")'
"#;
    let hits = scan_script(script, WorkingDir::Known(String::new()), &dirs);
    let paths: Vec<_> = hits
        .iter()
        .filter_map(|hit| match &hit.kind {
            ScriptHitKind::Invocation { path, .. } => Some(path.as_str()),
            ScriptHitKind::Unanalyzable { .. } => None,
        })
        .collect();
    assert_eq!(
        paths,
        vec![
            "scripts/build.sh",
            "scripts/lint.sh",
            "scripts/check.mts",
            "tools/ci/verify.ts",
            "scripts/nested.py",
            "scripts/sub.sh",
            "scripts/after-cd.sh",
        ]
    );
    assert!(hits
        .iter()
        .all(|hit| matches!(hit.kind, ScriptHitKind::Invocation { .. })));
}

#[test]
fn reports_dynamic_and_unterminated_scripts_as_unanalyzable() {
    let dirs = vec!["scripts".to_string()];
    for script in [
        r#""$RUNNER" arg"#,
        r#"eval "scripts/run.sh""#,
        r#"bash -c "$DYNAMIC_SCRIPT""#,
        r#"node "scripts/${TARGET}.mjs""#,
        r#"bash 'unterminated"#,
    ] {
        let hits = scan_script(script, WorkingDir::Known(String::new()), &dirs);
        assert!(
            hits.iter()
                .any(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. })),
            "expected unanalyzable for `{script}`, got {hits:?}"
        );
    }
}

#[test]
fn scans_subcommand_package_and_interpreter_runners() {
    let dirs = vec!["scripts".to_string()];
    let script = r#"
bun run --cwd . scripts/a.ts
bun x scripts/b.ts
bun --config bunfig.toml scripts/c.ts
bun -- scripts/d.ts
bun -e 'console.log("scripts/ignored.ts")'
bun --cwd .
deno run --allow-read scripts/e.ts
pwsh -File scripts/f.ps1
pwsh -WorkingDirectory . scripts/g.ps1
source scripts/h.sh
. scripts/i.sh
ruby -I lib scripts/j.rb
perl -I lib scripts/k.pl
npx -- scripts/l.mjs
uv run scripts/m.py
pnpm -C . exec node scripts/n.mjs
sh -ec 'bash scripts/o.sh'
"#;
    let hits = scan_script(script, WorkingDir::Known(String::new()), &dirs);
    let paths: Vec<_> = hits
        .iter()
        .filter_map(|hit| match &hit.kind {
            ScriptHitKind::Invocation { path, .. } => Some(path.as_str()),
            ScriptHitKind::Unanalyzable { .. } => None,
        })
        .collect();
    assert_eq!(
        paths,
        vec![
            "scripts/a.ts",
            "scripts/b.ts",
            "scripts/c.ts",
            "scripts/d.ts",
            "scripts/e.ts",
            "scripts/f.ps1",
            "scripts/g.ps1",
            "scripts/h.sh",
            "scripts/i.sh",
            "scripts/j.rb",
            "scripts/k.pl",
            "scripts/l.mjs",
            "scripts/m.py",
            "scripts/n.mjs",
            "scripts/o.sh",
        ]
    );
}

#[test]
fn parses_working_directory_values() {
    use super::workflow::parse_working_dir;
    assert_eq!(
        parse_working_dir(&Value::Null),
        WorkingDir::Known(String::new())
    );
    assert_eq!(
        parse_working_dir(&Value::String("./scripts/".to_string())),
        WorkingDir::Known("scripts".to_string())
    );
    assert_eq!(
        parse_working_dir(&Value::String("${{ inputs.dir }}".to_string())),
        WorkingDir::Dynamic("${{ inputs.dir }}".to_string())
    );
    assert_eq!(
        parse_working_dir(&Value::String("$WORK_DIR".to_string())),
        WorkingDir::Dynamic("$WORK_DIR".to_string())
    );
    assert_eq!(
        parse_working_dir(&Value::String("../outside".to_string())),
        WorkingDir::OutsideRepo
    );
    assert_eq!(
        parse_working_dir(&Value::Number(42.into())),
        WorkingDir::Dynamic("non-string".to_string())
    );
}

#[test]
fn scans_cd_and_dynamic_working_directory_paths() {
    let dirs = vec!["scripts".to_string(), "tools/ci".to_string()];
    let dynamic_hits = scan_script(
        "FOO=bar\nbash \"\"\nbash ./run.sh\nbash /abs/run.sh\nnode /abs/$X.js\nnode \"$SCRIPT\"\nnode \"tools/${TARGET}\"\ncd sub\n",
        WorkingDir::Dynamic("${{ inputs.dir }}".to_string()),
        &dirs,
    );
    assert!(
        dynamic_hits
            .iter()
            .filter(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. }))
            .count()
            >= 3,
        "expected dynamic dir/script unanalyzable hits, got {dynamic_hits:?}"
    );
    let known_empty_prefix = scan_script(
        "node \"$SCRIPT\"\ncat <<EOF\nunterminated",
        WorkingDir::Known(String::new()),
        &dirs,
    );
    assert_eq!(known_empty_prefix.len(), 1);
    assert_eq!(known_empty_prefix[0].token(), None);
    let known_dyn_token = scan_script("node \"$SCRIPT\"", WorkingDir::Known(String::new()), &dirs);
    assert_eq!(known_dyn_token[0].token(), Some("$SCRIPT"));
    assert_eq!(super::composites::normalize_local_uses("./${{ x }}"), None);
    let outside_hits = scan_script(
        "cd\n./run.sh\nnode \"$X\"\ncd sub\n",
        WorkingDir::Known("scripts".to_string()),
        &dirs,
    );
    assert!(outside_hits.is_empty(), "{outside_hits:?}");

    let cd_hits = scan_script(
        "cd \"$DIR\"\n./in-dyn.sh\ncd /tmp\n./in-tmp.sh\ncd -\n./in-prev.sh\n",
        WorkingDir::Known(String::new()),
        &dirs,
    );
    assert_eq!(cd_hits.len(), 2, "{cd_hits:?}");
    assert!(cd_hits
        .iter()
        .any(|hit| hit.token() == Some("./in-prev.sh")));

    let nested_prefix = scan_script(
        "node \"tools/${X}\"\nnode \"/abs/${X}\"\nbash -- scripts/dash.sh\nsh -o errexit -c 'bash scripts/opt.sh'\nsh --command 'bash scripts/cmd.sh'\nsh -ce 'ignored'\nbash -x\n",
        WorkingDir::Known(String::new()),
        &dirs,
    );
    assert!(nested_prefix.iter().any(|h| matches!(
        &h.kind,
        ScriptHitKind::Unanalyzable { token, .. } if token == "tools/${X}"
    )));
    let in_scripts = scan_script(
        "node \"${TARGET}.mjs\"",
        WorkingDir::Known("scripts".to_string()),
        &dirs,
    );
    assert_eq!(in_scripts.len(), 1, "{in_scripts:?}");
    assert_eq!(
        super::scripts::resolve_rel_path("", "https://example.com/a.sh"),
        None
    );
}

#[test]
fn checks_edge_cases_fixture_composites_include_and_unreadable_paths() {
    use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig, Unanalyzable,
    };
    use std::path::PathBuf;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-workflow-script-paths/edge-cases"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            message: None,
            options: serde_yaml::from_str(
                "scriptDirs: [scripts]\ninclude: ['.github/workflows/*.yml']\nunanalyzable: report\n",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let findings = super::check(&root, &config).unwrap();
    for needle in [
        "invalid workflow YAML",
        "non-string `run` value",
        "dynamic local action",
        "could not be read",
        "containing invalid YAML",
        "scripts/dup.sh",
        "scripts/from-composite.sh",
        "scripts/deep.sh",
        "via `.github/actions/opaque/action.yml`) has a non-string `run` value",
        "via `.github/actions/opaque/action.yml`) uses dynamic local action",
        "Update on.pull_request.paths, on.push.paths-ignore",
    ] {
        assert!(
            findings.iter().any(|f| f.message.contains(needle)),
            "missing `{needle}` in findings: {findings:?}"
        );
    }

    let custom_msg_config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            message: Some("Custom remediation hint.".into()),
            options: serde_yaml::from_str("scriptDirs: [scripts]\nunanalyzable: report\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let missing_file = root.join(".github/workflows/does-not-exist.yml");
    let edge_file = root.join(".github/workflows/edge.yml");
    let files = vec![edge_file, missing_file];
    let with_missing = super::check_with_files(&root, &custom_msg_config, &files).unwrap();
    assert!(with_missing
        .iter()
        .any(|f| f.message.contains("Custom remediation hint.")));
    assert!(with_missing
        .iter()
        .any(|f| f.message.contains("cannot read workflow as UTF-8")));

    let parsed = parse_triggers(
        "on:\n  pull_request:\n    paths:\n      - 'apps/**'\n",
        &serde_yaml::from_str("on:\n  pull_request:\n    paths: ['apps/**']\n").unwrap(),
        &["pull_request".to_string()],
    );
    let hits = scan_script(
        "./scripts/fallback.sh",
        WorkingDir::Known(String::new()),
        &["scripts".to_string()],
    );
    let mut diag = AnalysisDiagnostics::default();
    let emit_ctx = super::emit::EmitCtx {
        rel: ".github/workflows/flow.yml",
        source: "jobs:\n  a:\n    steps:\n      - run: ./scripts/fallback.sh\n",
        filters: &parsed.filters,
        unanalyzable: Unanalyzable::Report,
        rule_message: None,
    };
    super::emit::emit_hits(
        &emit_ctx,
        super::emit::Origin::Run {
            place: "job `a` step 1",
            located: None,
            script: "./scripts/fallback.sh",
        },
        &hits,
        &mut diag,
    );
    assert_eq!(diag.findings.len(), 1);
    assert_eq!(diag.findings[0].line, 4);

    let mut fallback_diag = AnalysisDiagnostics::default();
    let mismatch_located = crate::codebase::rules::github_actions_script_expressions::StepRun {
        job: "a".to_string(),
        step: 1,
        item_line: 8,
        lines: vec![(9, "echo unrelated".to_string())],
    };
    super::emit::emit_hits(
        &emit_ctx,
        super::emit::Origin::Run {
            place: "job `a` step 1",
            located: Some(&mismatch_located),
            script: "./scripts/fallback.sh\necho done",
        },
        &hits,
        &mut fallback_diag,
    );
    assert_eq!(fallback_diag.findings[0].line, 9);

    let main_fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-workflow-script-paths/fixture"),
    );
    let main_config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("scriptDirs: [scripts, tools/ci]\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(super::check(&main_fixture.join("compliant"), &main_config)
        .unwrap()
        .is_empty());
    assert!(!super::check(&main_fixture.join("reported"), &main_config)
        .unwrap()
        .is_empty());
    assert!(
        super::check(&main_fixture.join("unanalyzable"), &main_config)
            .unwrap()
            .is_empty()
    );

    let empty_dirs = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("scriptDirs: []\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(super::check(&main_fixture, &empty_dirs).unwrap().is_empty());

    let compiled = compile_yaml("scriptDirs: [scripts]\n").unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let run_ctx = super::diagnostics::RunCtx {
        root: &main_fixture,
        sources: &sources,
        compiled: &compiled,
        unanalyzable: Unanalyzable::Report,
        rule_message: None,
        defer_suppression: false,
    };
    let no_jobs = super::workflow::check_source(
        &run_ctx,
        &main_fixture.join(".github/workflows/no-jobs.yml"),
        ".github/workflows/no-jobs.yml",
        "on:\n  pull_request:\n    paths: ['apps/**']\n",
    );
    assert!(no_jobs.findings.is_empty());
    let bad_trigger = super::workflow::check_source(
        &run_ctx,
        &main_fixture.join(".github/workflows/bad-trigger.yml"),
        ".github/workflows/bad-trigger.yml",
        "on: 42\njobs: {}\n",
    );
    assert_eq!(bad_trigger.findings.len(), 1);
}
