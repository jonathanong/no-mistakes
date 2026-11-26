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
    ] {
        let value: Value = serde_yaml::from_str(bad).unwrap();
        let events = vec!["push".to_string(), "pull_request".to_string()];
        let parsed = parse_triggers(bad, &value, &events);
        assert!(
            !parsed.unanalyzable.is_empty(),
            "expected unanalyzable trigger for:\n{bad}"
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
