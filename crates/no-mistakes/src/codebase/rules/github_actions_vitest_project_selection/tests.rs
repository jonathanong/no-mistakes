use super::judge::narrowings;
use super::options::{compile, NameFilters};
use super::{Options, ProjectOption};
use std::collections::BTreeMap;

fn policies<'a>(pairs: &'a [(&'a str, NameFilters)]) -> BTreeMap<&'a str, NameFilters> {
    pairs.iter().copied().collect()
}

fn arguments(script: &str, pairs: &[(&str, NameFilters)]) -> Vec<String> {
    narrowings(script, &policies(pairs))
        .into_iter()
        .map(|hit| hit.argument)
        .collect()
}

fn widgets() -> Vec<(&'static str, NameFilters)> {
    vec![("widgets", NameFilters::Allow)]
}

#[test]
fn file_positionals_are_rejected_and_flags_are_not() {
    let policies = widgets();
    assert_eq!(
        arguments(
            "pnpm exec vitest run --project widgets src/widgets.test.ts",
            &policies
        ),
        ["src/widgets.test.ts"]
    );
    assert!(arguments("pnpm exec vitest run --project widgets", &policies).is_empty());
    assert!(arguments("vitest run --project widgets --reporter=dot", &policies).is_empty());
    assert!(arguments(
        "pnpm exec vitest run --project widgets --config vitest.config.ts",
        &policies
    )
    .is_empty());
    assert!(arguments("pnpm exec vitest run --project widgets widgets", &policies).is_empty());
    assert!(arguments(
        "pnpm exec vitest run --project orders src/orders.test.ts",
        &policies
    )
    .is_empty());
    assert!(arguments("node scripts/build.mjs src/widgets.test.ts", &policies).is_empty());
}

#[test]
fn name_filters_follow_the_project_policy() {
    let reject = [("items", NameFilters::Reject)];
    assert_eq!(
        arguments("pnpm exec vitest run --project items items", &reject),
        ["items"]
    );
    assert_eq!(
        arguments("pnpm exec vitest run --project items -t items", &reject),
        ["items"]
    );
    assert_eq!(
        arguments("vitest run --project=items src/items.test.ts", &reject),
        ["src/items.test.ts"]
    );
    assert!(arguments(
        "pnpm --filter widgets exec vitest run --project widgets",
        &widgets()
    )
    .is_empty());
    assert_eq!(
        arguments(
            "echo ok && pnpm exec vitest run --project widgets src/widgets.test.ts",
            &widgets()
        ),
        ["src/widgets.test.ts"]
    );
    assert!(arguments(
        "pnpm exec vitest run --project widgets $WIDGETS",
        &widgets()
    )
    .is_empty());
    assert!(arguments(
        "pnpm exec vitest run --project widgets # src/widgets.test.ts",
        &widgets()
    )
    .is_empty());
}

#[test]
fn wrappers_quotes_and_bare_files_follow_the_command() {
    let allow = widgets();
    let reject = [("items", NameFilters::Reject)];
    let file = ["src/widgets.test.ts"];
    assert_eq!(
        arguments(
            "( { vitest run --project widgets src/widgets.test.ts; } )",
            &allow
        ),
        file
    );
    assert_eq!(
        arguments(
            "pnpm -- vitest run --project widgets src/widgets.test.ts",
            &allow
        ),
        file
    );
    assert!(arguments("pnpm exec node", &allow).is_empty());
    assert!(arguments("pnpm --filter widgets", &allow).is_empty());
    assert_eq!(
        arguments(
            "sudo -n vitest run --project widgets src/widgets.test.ts",
            &allow
        ),
        file
    );
    for script in [
        "npm --prefix widgets exec vitest run --project widgets src/widgets.test.ts",
        "npx -p vitest vitest run --project widgets src/widgets.test.ts",
        "yarn --cwd widgets exec vitest run --project widgets src/widgets.test.ts",
        "bun --filter widgets x vitest run --project widgets src/widgets.test.ts",
        "bunx --cwd widgets vitest run --project widgets src/widgets.test.ts",
    ] {
        assert_eq!(arguments(script, &allow), file, "{script}");
    }
    assert_eq!(
        arguments(
            ";; vitest run --project widgets \\\nsrc/wid\\gets.test.ts",
            &allow
        ),
        file
    );
    assert!(arguments("vitest \\", &allow).is_empty());
    assert_eq!(
        arguments(
            "vitest run --project widgets 'src/widgets.test.ts\n'",
            &allow
        ),
        ["src/widgets.test.ts\n"]
    );
    assert_eq!(
        arguments(
            "vitest run --project widgets \"src/widgets.test.ts\n\"",
            &allow
        ),
        ["src/widgets.test.ts\n"]
    );
    assert!(arguments("vitest run --project widgets \"src/$WIDGETS\"", &allow).is_empty());
    assert!(arguments("vitest run --project items -t=", &reject).is_empty());
    assert!(arguments("vitest run --project items -t", &reject).is_empty());
    assert!(arguments("vitest run --project", &allow).is_empty());
    assert_eq!(
        arguments("vitest run --project widgets widgets.test.ts", &allow),
        ["widgets.test.ts"]
    );
}

#[test]
fn options_require_unique_projects_and_known_name_filters() {
    let empty = Options::default();
    assert!(compile(&empty).unwrap().projects.is_empty());
    let duplicated = Options {
        projects: vec![
            ProjectOption {
                project: "widgets".to_string(),
                workflows: vec![".github/workflows/widgets.yml".to_string()],
                name_filters: None,
            },
            ProjectOption {
                project: "widgets".to_string(),
                workflows: vec![".github/workflows/items.yml".to_string()],
                name_filters: Some("reject".to_string()),
            },
        ],
    };
    assert!(compile(&duplicated)
        .unwrap_err()
        .to_string()
        .contains("duplicated"));
    let unknown = Options {
        projects: vec![ProjectOption {
            project: "items".to_string(),
            workflows: vec![".github/workflows/items.yml".to_string()],
            name_filters: Some("ignore".to_string()),
        }],
    };
    assert!(compile(&unknown)
        .unwrap_err()
        .to_string()
        .contains("nameFilters"));
    let empty_project = Options {
        projects: vec![ProjectOption {
            project: " ".to_string(),
            workflows: vec![".github/workflows/widgets.yml".to_string()],
            name_filters: None,
        }],
    };
    assert!(compile(&empty_project)
        .unwrap_err()
        .to_string()
        .contains("project"));
    let empty_workflows = Options {
        projects: vec![ProjectOption {
            project: "widgets".to_string(),
            workflows: Vec::new(),
            name_filters: None,
        }],
    };
    assert!(compile(&empty_workflows)
        .unwrap_err()
        .to_string()
        .contains("workflows"));
    let escaped = Options {
        projects: vec![ProjectOption {
            project: "widgets".to_string(),
            workflows: vec!["../widgets.yml".to_string()],
            name_filters: None,
        }],
    };
    assert!(compile(&escaped)
        .unwrap_err()
        .to_string()
        .contains("relative"));
    let allow = compile(&Options {
        projects: vec![ProjectOption {
            project: " widgets ".to_string(),
            workflows: vec![" ./.github/workflows/widgets.yml ".to_string()],
            name_filters: Some(" allow ".to_string()),
        }],
    })
    .unwrap();
    assert_eq!(allow.projects[0].name_filters, NameFilters::Allow);
    assert_eq!(allow.projects[0].project, "widgets");
}

#[test]
fn launchers_quotes_and_name_filters_follow_the_command() {
    let allow = widgets();
    let reject = [("items", NameFilters::Reject)];
    assert_eq!(
        described(
            "vitest run --project items --testNamePattern orders",
            &reject
        ),
        ["items:true:orders"]
    );
    assert_eq!(
        described("vitest run --project items -t=orders", &reject),
        ["items:true:orders"]
    );
    assert_eq!(
        described(
            "vitest run --project widgets -- src/widgets.test.ts",
            &allow
        ),
        ["widgets:false:src/widgets.test.ts"]
    );
    assert_eq!(
        described("vitest --foo src/widgets.test.ts --project widgets", &allow),
        ["widgets:false:src/widgets.test.ts"]
    );
    assert!(described("vitest run --project widgets --dir src/widgets", &allow).is_empty());
    assert!(described("vitest run --project widgets --config", &allow).is_empty());
    assert_eq!(
        described(
            "sudo env FOO=bar pnpm run vitest --project widgets src/widgets.test.ts",
            &allow
        ),
        ["widgets:false:src/widgets.test.ts"]
    );
    assert!(described(
        "pnpm run test --project widgets src/widgets.test.ts",
        &allow
    )
    .is_empty());
    assert_eq!(
        described(
            "echo a | pnpm exec vitest run --project widgets 'src/widgets.test.ts'",
            &allow
        ),
        ["widgets:false:src/widgets.test.ts"]
    );
    let both = [
        ("widgets", NameFilters::Allow),
        ("items", NameFilters::Reject),
    ];
    assert_eq!(
        described(
            "vitest run --project widgets --project items src/widgets.test.ts orders",
            &both
        ),
        ["widgets:false:src/widgets.test.ts", "items:false:orders"]
    );
}

fn described(script: &str, pairs: &[(&str, NameFilters)]) -> Vec<String> {
    narrowings(script, &policies(pairs))
        .into_iter()
        .map(|hit| format!("{}:{}:{}", hit.project, hit.flag, hit.argument))
        .collect()
}

#[test]
fn workflow_scan_reports_configured_files_only() {
    let compiled = compile(&Options {
        projects: vec![ProjectOption {
            project: "widgets".to_string(),
            workflows: vec![
                ".github/workflows/widgets.yml".to_string(),
                ".github/workflows/broken.yml".to_string(),
                ".github/actions/widgets/action.yml".to_string(),
                ".github/actions/orders/action.yml".to_string(),
            ],
            name_filters: Some("allow".to_string()),
        }],
    })
    .unwrap();
    assert!(super::runs::check_source("README.md", "jobs: {}", &compiled).is_empty());
    assert!(super::runs::check_source(
        ".github/workflows/orders.yml",
        "name: Orders\non: push\njobs:\n  unit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: pnpm exec vitest run --project widgets src/orders.test.ts\n",
        &compiled,
    )
    .is_empty());
    assert!(
        super::runs::check_source(".github/workflows/broken.yml", "jobs: [", &compiled).is_empty()
    );
    assert!(super::runs::check_source(
        ".github/actions/orders/action.yml",
        "name: Orders\ndescription: orders\nruns:\n  using: node20\n  main: index.js\n",
        &compiled,
    )
    .is_empty());
    assert!(super::runs::check_source(
        ".github/actions/widgets/action.yml",
        "runs:\n  using: composite\n",
        &compiled,
    )
    .is_empty());
    let composite = "name: Widgets\ndescription: widgets\nruns:\n  using: composite\n  steps:\n    - run: pnpm exec vitest run --project widgets src/widgets.test.ts\n";
    let hits =
        super::runs::check_source(".github/actions/widgets/action.yml", composite, &compiled);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("composite step 1"), "{hits:?}");
    assert_eq!(hits[0].target.as_deref(), Some("src/widgets.test.ts"));
    let workflow = "name: Widgets\non: push\njobs:\n  skipped: 1\n  unit:\n    steps: 1\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: 1\n      - run: pnpm exec vitest run --project widgets src/widgets.test.ts\n";
    let hits = super::runs::check_source(".github/workflows/widgets.yml", workflow, &compiled);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].message.contains("job `build` step 2"), "{hits:?}");
    let quoted = "name: Widgets\non: push\njobs:\n  unit:\n    runs-on: ubuntu-latest\n    steps:\n      - run: \"pnpm exec vitest run --project widgets src/widgets.test.ts\\necho done\"\n";
    let hits = super::runs::check_source(".github/workflows/widgets.yml", quoted, &compiled);
    assert_eq!(hits.len(), 1, "{hits:?}");
    let flow = "name: Widgets\non: push\njobs:\n  unit:\n    runs-on: ubuntu-latest\n    steps: [{run: pnpm exec vitest run --project widgets src/widgets.test.ts}]\n";
    let hits = super::runs::check_source(".github/workflows/widgets.yml", flow, &compiled);
    assert_eq!(hits.len(), 1, "{hits:?}");
}

#[test]
fn yaml_lines_follow_the_located_script() {
    use crate::codebase::rules::github_actions_script_expressions::StepRun;
    use crate::codebase::rules::github_actions_workflow_files::run_scripts::scalar_name;
    let located = StepRun {
        job: "unit".to_string(),
        step: 1,
        item_line: 4,
        lines: vec![(8, "pnpm exec vitest".to_string())],
    };
    assert_eq!(super::runs::yaml_line(None, "pnpm", 1, "pnpm"), 1);
    assert_eq!(
        super::runs::yaml_line(
            Some(&StepRun {
                job: "unit".to_string(),
                step: 1,
                item_line: 4,
                lines: Vec::new(),
            }),
            "pnpm",
            1,
            "pnpm"
        ),
        1
    );
    assert_eq!(super::runs::yaml_line(Some(&located), "pnpm", 1, "pnpm"), 8);
    assert_eq!(super::runs::yaml_line(Some(&located), "pnpm", 3, "pnpm"), 8);
    let split = StepRun {
        job: "unit".to_string(),
        step: 1,
        item_line: 4,
        lines: vec![
            (8, "echo done".to_string()),
            (9, "src/widgets.test.ts".to_string()),
        ],
    };
    assert_eq!(
        super::runs::yaml_line(
            Some(&split),
            "src/widgets.test.ts",
            1,
            "src/widgets.test.ts"
        ),
        9
    );
    assert_eq!(
        super::runs::yaml_line(Some(&split), "src/widgets.test.ts", 1, "missing"),
        8
    );
    assert_eq!(
        scalar_name(&serde_yaml::Value::String("unit".to_string())),
        "unit"
    );
    assert_eq!(
        scalar_name(&serde_yaml::Value::String(String::new())),
        "job"
    );
    assert_eq!(scalar_name(&serde_yaml::Value::from(1_i64)), "1");
    assert_eq!(scalar_name(&serde_yaml::Value::Bool(true)), "true");
    assert_eq!(scalar_name(&serde_yaml::Value::Null), "job");
}
