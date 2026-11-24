use super::{check, compile_options, scan, CompiledOptions, Options, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn compile_with(yaml: &str, root_yaml: &str) -> CompiledOptions {
    let config: NoMistakesConfig = serde_yaml::from_str(root_yaml).expect("parse config");
    let opts: Options = serde_yaml::from_str(yaml).expect("parse options");
    compile_options(opts, &config).expect("compile options")
}

#[test]
fn detects_serial_mode_and_config_flags() {
    let opts = compile_with("{}", "version: 2\n");
    let config_src = r#"
import { defineConfig } from "@playwright/test";
export default defineConfig({
  "workers": process.env.CI ? 1 : undefined,
  fullyParallel: process.env.CI || false,
  projects: [
    { name: `chromium`, workers: 2, fullyParallel: true },
    { name: "serial-suite", workers: 1, fullyParallel: false },
  ],
});
"#;
    let findings = scan::scan_source("playwright.config.ts", config_src, &opts);
    assert_eq!(findings.len(), 4, "findings: {findings:#?}");
    assert!(findings.iter().all(|f| f.rule == RULE_ID));

    let test_src = r#"
import { test } from "@playwright/test";
test.describe.configure({ mode: "serial" });
test.describe.configure({ mode: "parallel" });
test.describe.serial("ordered flow", () => {});
test.describe.serial.only("focused", () => {});
"#;
    let test_findings = scan::scan_source("e2e/app.spec.ts", test_src, &opts);
    assert_eq!(test_findings.len(), 3, "findings: {test_findings:#?}");
}

#[test]
fn allowlist_exempts_matching_paths_and_projects() {
    let opts = compile_with(
        r#"
allow:
  - project: serial-suite
    reason: shares external test sandbox
  - path: "e2e/legacy/*.spec.ts"
    reason: legacy checkout flow under migration
"#,
        "version: 2\n",
    );
    let config_src = r#"
module.exports = {
  projects: [
    { name: "serial-suite", workers: 1, fullyParallel: false },
    { name: "default", workers: 1 },
  ],
};
"#;
    let findings = scan::scan_source("playwright.config.js", config_src, &opts);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("default"));

    let test_src = r#"test.describe.serial("legacy", () => {});"#;
    assert!(scan::scan_source("e2e/legacy/flow.spec.ts", test_src, &opts).is_empty());
    assert_eq!(
        scan::scan_source("e2e/new/flow.spec.ts", test_src, &opts).len(),
        1
    );
}

#[test]
fn supports_custom_config_factories_cycles_and_standalone_check() {
    let opts = compile_with(
        "configs: ['configs/pw.ts']\n",
        "version: 2\ntests:\n  playwright:\n    configs: ['custom/pw-runner.ts']\n",
    );
    let arrow = "const cfg = () => ({ workers: 1 });\nexport default cfg;\n";
    let block_arrow = "export default () => { return { workers: 1 }; };\n";
    let fn_expr = "export default function () { return { workers: 1 }; };\n";
    let fn_const = "const mk = function () { return { workers: 1 }; };\nexport default mk;\n";
    let cycle = "const a = a;\nexport default a;\n";
    assert_eq!(scan::scan_source("configs/pw.ts", arrow, &opts).len(), 1);
    assert_eq!(
        scan::scan_source("custom/pw-runner.ts", block_arrow, &opts).len(),
        1
    );
    assert_eq!(scan::scan_source("configs/pw.ts", fn_expr, &opts).len(), 1);
    assert_eq!(scan::scan_source("configs/pw.ts", fn_const, &opts).len(), 1);
    assert!(scan::scan_source("configs/pw.ts", cycle, &opts).is_empty());
    assert!(scan::scan_source("src/other.ts", arrow, &opts).is_empty());

    let fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/playwright-no-serial/fixture/reported"),
    );
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
version: 2
rules:
  - rule: playwright-no-serial
    scope: repository
    message: "policy:"
    options:
      files: ["e2e/checkout.spec.ts"]
"#,
    )
    .unwrap();
    let findings = check(&fixture, &cfg).expect("standalone check");
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().all(|f| f.message.starts_with("policy: ")));
}

#[test]
fn rejects_invalid_allow_entries() {
    let config = NoMistakesConfig::default();
    let missing_reason: Options =
        serde_yaml::from_str("allow:\n  - path: 'e2e/**'\n    reason: '   '\n").unwrap();
    assert!(compile_options(missing_reason, &config).is_err());

    let missing_selector: Options =
        serde_yaml::from_str("allow:\n  - reason: 'needs selector'\n").unwrap();
    assert!(compile_options(missing_selector, &config).is_err());
}

#[test]
fn covers_non_matching_ast_and_config_edge_cases() {
    let opts = compile_with("{}", "version: 2\n");
    let edge_cfg = r#"
const base = {};
foo.bar = 1;
export default {
  ...base,
  0: 1,
  get workers() { return 1; },
  workers: 1.5,
  fullyParallel: "no",
  projects: [
    otherVar,
    { name: `dyn-${x}`, workers: 1, fullyParallel: false },
    { name: dynamicName, workers: 2 },
  ],
};
"#;
    let findings = scan::scan_source("playwright.config.ts", edge_cfg, &opts);
    assert_eq!(findings.len(), 2);

    assert!(scan::scan_source("playwright.config.ts", "export default 42;", &opts).is_empty());
    assert!(scan::scan_source(
        "playwright.config.ts",
        "export default () => { return; };",
        &opts
    )
    .is_empty());
    assert!(scan::scan_source(
        "playwright.config.ts",
        "export default () => { const x = 1; };",
        &opts
    )
    .is_empty());
    assert!(
        scan::scan_source("playwright.config.ts", "export default class Foo {}", &opts).is_empty()
    );
    assert!(scan::scan_source(
        "playwright.config.ts",
        "let uninit; export default uninit;",
        &opts
    )
    .is_empty());
    let allowed_cfg = compile_with(
        "allow:\n  - path: 'playwright.config.ts'\n    reason: temp\n",
        "version: 2\n",
    );
    assert!(scan::scan_source(
        "playwright.config.ts",
        "export default { workers: 1 };",
        &allowed_cfg
    )
    .is_empty());
    assert!(scan::scan_source(
        "playwright.config.ts",
        "export default { projects: getProjects() };",
        &opts
    )
    .is_empty());

    let edge_spec = r#"
test?.describe.serial("opt2", () => {});
getFn()("call");
test.describe.configure();
test.describe.configure(cfg);
test.describe.configure({ retries: 0 });
"#;
    assert!(scan::scan_source("e2e/edge.spec.ts", edge_spec, &opts).is_empty());
    assert_eq!(
        scan::scan_source(
            "e2e/opt.spec.ts",
            r#"import { test } from "@playwright/test"; test.describe.serial?.("opt", () => {});"#,
            &opts
        )
        .len(),
        1
    );
    let aliased_pw = r#"
import { test as pw } from "@playwright/test";
const test = { describe: { serial: () => {} } };
test.describe.serial("unrelated", () => {});
pw.describe.serial("real", () => {});
pw.describe.configure({ mode: "serial" });
"#;
    assert_eq!(
        scan::scan_source("e2e/aliased.spec.ts", aliased_pw, &opts).len(),
        2
    );
    assert!(scan::scan_source(
        "unit/vitest.test.ts",
        r#"import { test } from "vitest"; test.describe.serial("not pw", () => {});"#,
        &opts
    )
    .is_empty());

    let short_circuit = r#"
export default defineConfig(ci ? { workers: 1 } : { workers: 4 });
"#;
    assert_eq!(
        scan::scan_source("playwright.config.ts", short_circuit, &opts).len(),
        1
    );
    let local_shadow = r#"
const cfg = { workers: 1 };
export default () => { const cfg = { workers: 4 }; return cfg; };
"#;
    assert!(scan::scan_source("playwright.config.ts", local_shadow, &opts).is_empty());
    let logical_sc = r#"
export default {
  fullyParallel: true || false,
  projects: [
    { name: "and-false", fullyParallel: process.env.CI && false },
    { name: "and-true", fullyParallel: true && true },
    { name: "and-lit-false", fullyParallel: false && true },
    { name: "or-lit-false", fullyParallel: false || true },
    { name: "nullish-null", workers: null ?? 4 },
    { name: "nullish-val", workers: 4 ?? 1 },
  ],
};
"#;
    assert_eq!(
        scan::scan_source("playwright.config.ts", logical_sc, &opts).len(),
        2
    );
    let win_opts = compile_with(
        "{}",
        "version: 2\ntests:\n  playwright:\n    configs: ['.\\configs\\pw.ts']\n",
    );
    assert_eq!(
        scan::scan_source("configs/pw.ts", "export default { workers: 1 };", &win_opts).len(),
        1
    );
    let empty_store = crate::codebase::rules::source_store_for_files(&[]);
    assert!(scan::scan_file(
        std::path::Path::new("."),
        std::path::Path::new("README.md"),
        &opts,
        &empty_store
    )
    .is_empty());
    assert!(scan::scan_file(
        std::path::Path::new("."),
        std::path::Path::new("missing.spec.ts"),
        &opts,
        &empty_store
    )
    .is_empty());
}
