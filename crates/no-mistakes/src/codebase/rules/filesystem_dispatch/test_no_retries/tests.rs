use super::{check, options, scan_ci, scan_ts, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn compile_with(yaml: &str, root_yaml: &str) -> options::CompiledOptions {
    let config: NoMistakesConfig = serde_yaml::from_str(root_yaml).expect("parse config");
    let opts: options::Options = serde_yaml::from_str(yaml).expect("parse options");
    options::compile(opts, &config).expect("compile options")
}

#[test]
fn detects_vitest_playwright_and_jest_retries() {
    let opts = compile_with("{}", "version: 2\n");
    let vitest_cfg = r#"
import { defineConfig } from "vitest/config";
export default defineConfig({
  test: {
    retry: 2,
    projects: [
      { test: { name: "unit", retry: { count: 1 } } },
      { test: { name: "clean", retry: 0 } },
    ],
  },
});
"#;
    let vitest_findings = scan_ts::scan_source("vitest.config.ts", vitest_cfg, &opts);
    assert_eq!(vitest_findings.len(), 2, "{vitest_findings:#?}");
    assert!(vitest_findings.iter().all(|f| f.rule == RULE_ID));

    let pw_cfg = r#"
export default {
  retries: process.env.CI ?? 2,
  projects: [{ name: "chromium", retries: 1 }],
};
"#;
    let pw_findings = scan_ts::scan_source("playwright.config.ts", pw_cfg, &opts);
    assert_eq!(pw_findings.len(), 2, "{pw_findings:#?}");

    let spec_src = r#"
jest.retryTimes(3);
jest.retryTimes(0);
test.describe.configure({ retries: 2 });
test.describe.configure({ retries: 0 });
test("a", { retry: 2 }, () => {});
it.concurrent("b", () => {}, { retry: { count: 1 } });
describe("c", { retry: 1 }, () => {});
test.each([1])("d %i", { retry: 2 }, () => {});
"#;
    let spec_findings = scan_ts::scan_source("src/example.test.ts", spec_src, &opts);
    assert_eq!(spec_findings.len(), 6, "{spec_findings:#?}");
}

#[test]
fn respects_max_retries_and_allowlist() {
    let opts = compile_with(
        r#"
maxRetries: 1
allow:
  - project: smoke
    reason: external staging rate limits
  - path: "tests/legacy/**"
    reason: legacy integration suite
"#,
        "version: 2\n",
    );
    let pw_cfg = r#"
export default {
  retries: 1,
  projects: [
    { name: "smoke", retries: 3 },
    { name: "core", retries: 2 },
  ],
};
"#;
    let findings = scan_ts::scan_source("playwright.config.ts", pw_cfg, &opts);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("core"));

    assert!(
        scan_ts::scan_source("tests/legacy/a.test.ts", "jest.retryTimes(5);", &opts).is_empty()
    );
}

#[test]
fn detects_ci_retry_actions_and_scoped_shell_reruns() {
    let workflow = r#"
name: CI
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: nick-fields/retry@v3
        with:
          max_attempts: 3
          command: pnpm exec vitest run
      - uses: nick-fields/retry@v3
        with:
          max_attempts: 1
          command: pnpm exec vitest run
      - uses: custom-org/retry-step@v1
      - run: |
          # retry loop with quotes and escape
          for i in '1' "2 \"3\""; do \
            ENV=1 pnpm exec vitest run && break
          done
      - run: pnpm exec playwright test || pnpm exec playwright test
      - run: pnpm exec vitest run || echo "failed"
      - run: |
          for i in 1 2 3; do
            echo "not a test"
          done
"#;
    let no_runners = compile_with("retryActions: ['custom-org/retry-step']\n", "version: 2\n");
    let action_only =
        scan_ci::scan_source(".github/workflows/ci.yml", workflow, false, &no_runners);
    assert_eq!(action_only.len(), 2, "{action_only:#?}");

    let with_runners = compile_with(
        "retryActions: ['custom-org/retry-step']\n",
        r#"
version: 2
tests:
  vitest:
    configs: ["vitest.config.ts"]
  playwright:
    configs: ["playwright.config.ts"]
"#,
    );
    let all = scan_ci::scan_source(".github/workflows/ci.yml", workflow, false, &with_runners);
    assert_eq!(all.len(), 4, "{all:#?}");
}

#[test]
fn derives_ci_commands_from_all_configured_runners_and_runs_standalone() {
    let opts = compile_with(
        "ciTestCommands: ['pnpm test']\n",
        r#"
version: 2
tests:
  jest:
    configs: ["jest.config.ts"]
  rust:
    packages: ["."]
  go:
    modules: ["."]
  python:
    packages: ["."]
  swift:
    packages: ["."]
    baseCommands:
      ".": ["swift", "test", "--parallel"]
  dotnet:
    solutions: ["App.sln"]
  rails:
    apps: ["web"]
  php:
    apps: ["api"]
"#,
    );
    assert!(opts.ci_test_commands.contains(&vec!["jest".into()]));
    assert!(opts
        .ci_test_commands
        .contains(&vec!["go".into(), "test".into()]));
    assert!(opts.ci_test_commands.contains(&vec!["pytest".into()]));
    assert!(opts.ci_test_commands.contains(&vec![
        "swift".into(),
        "test".into(),
        "--parallel".into()
    ]));
    assert!(opts
        .ci_test_commands
        .contains(&vec!["dotnet".into(), "test".into()]));
    assert!(opts.ci_test_commands.contains(&vec!["rspec".into()]));
    assert!(opts.ci_test_commands.contains(&vec!["phpunit".into()]));
    assert!(opts
        .ci_test_commands
        .contains(&vec!["pnpm".into(), "test".into()]));

    let composite = r#"
name: Run tests
runs:
  using: composite
  steps:
    - uses: wandalen/wretry.action@v3
      with:
        attempt_limit: "2"
    - run: until cargo test; do sleep 1; done
"#;
    let findings = scan_ci::scan_source(".github/actions/run/action.yml", composite, true, &opts);
    assert_eq!(findings.len(), 2, "{findings:#?}");

    let fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-no-retries/fixture/reported"),
    );
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
version: 2
rules:
  - rule: test-no-retries
    scope: repository
    options:
      files: ["tests/legacy/**"]
"#,
    )
    .unwrap();
    assert_eq!(check(&fixture, &cfg).expect("standalone check").len(), 1);

    let config = NoMistakesConfig::default();
    let bad_action: options::Options = serde_yaml::from_str("retryActions: ['  ']\n").unwrap();
    assert!(options::compile(bad_action, &config).is_err());
    let bad_cmd: options::Options = serde_yaml::from_str("ciTestCommands: ['   ']\n").unwrap();
    assert!(options::compile(bad_cmd, &config).is_err());
}
