use super::super::policy;
use super::super::{check_with_files, RuleFinding};
use super::{load_config, parsed, rule_root, POLICY};
use serde_yaml::Value;

fn run_file(case: &str, relative: &str) -> Vec<RuleFinding> {
    let root = rule_root().join("fixture").join(case);
    let file = root.join(relative);
    check_with_files(&root, &load_config("policy.yml"), &[file]).unwrap()
}

#[test]
fn remote_families_fold_owner_and_repo_only() {
    assert_eq!(
        policy::family_of("Example/Publish-Widget@V1"),
        Some("example/publish-widget".to_string())
    );
    assert_eq!(
        policy::family_of("Example/Publish-Widget/Restore"),
        Some("example/publish-widget/Restore".to_string())
    );
    assert_eq!(policy::family_of("Widget"), Some("Widget".to_string()));
    assert_eq!(
        policy::family_of("./.github/actions/Publish-Widget"),
        Some("./.github/actions/Publish-Widget".to_string())
    );
    assert_eq!(
        policy::family_of("/opt/actions/Publish-Widget"),
        Some("/opt/actions/Publish-Widget".to_string())
    );
    assert_eq!(
        policy::family_of("docker://Orders:1"),
        Some("docker://Orders:1".to_string())
    );
}

#[test]
fn case_variants_merge_and_subpaths_stay_distinct() {
    let merged = r#"
actions:
  - uses: Example/Publish-Widget
    require:
      keep-days: 1
  - uses: example/publish-widget@v1
    require:
      keep-days: 1
"#;
    let compiled = policy::compile(serde_yaml::from_str(merged).unwrap()).unwrap();
    assert_eq!(compiled.actions.len(), 1);
    assert_eq!(compiled.actions[0].name, "example/publish-widget");
    let conflict = r#"
actions:
  - uses: Example/Publish-Widget
    require:
      keep-days: 1
  - uses: example/publish-widget@v1
    require:
      keep-days: 2
"#;
    let error = policy::compile(serde_yaml::from_str(conflict).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("keep-days"), "{error}");
    let source = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
      - uses: example/publish-widget/restore@v1
        with:
          keep-days: 30
      - uses: Example/Publish-Widget/Restore@v1
"#;
    let options = r#"
actions:
  - uses: Example/Publish-Widget
    require:
      keep-days: 1
  - uses: example/publish-widget/Restore
    require:
      keep-days: 1
"#;
    let findings = parsed(source, options);
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().any(|finding| {
        finding.message.contains("calls `example/publish-widget`")
            && !finding.message.contains("Restore")
    }));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("example/publish-widget/Restore")));
}

#[test]
fn expression_text_mismatches_a_required_scalar() {
    let source = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          keep-days: ${{ inputs.days }}
          replace: true
      - uses: ${{ matrix.widget }}
        with:
          keep-days: 30
          replace: false
"#;
    let findings = parsed(source, POLICY);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("${{ inputs.days }}"));
}

#[test]
fn composite_steps_report_on_the_step_label() {
    let source = r#"
runs:
  using: composite
  steps:
    - nope
    - run: echo widgets
    - name: publish widgets
      uses: example/publish-widget@v1
      with:
        keep-days: 30
        replace: true
"#;
    let findings = parsed(source, POLICY);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("composite step \"publish widgets\""));
    assert_eq!(findings[0].target.as_deref(), Some("publish widgets"));
    assert_eq!(findings[0].line, 10);
    for source in [
        "runs:\n  using: node20\n  main: index.js\n",
        "runs:\n  using: composite\n",
        "runs:\n  using: composite\n  steps: nope\n",
    ] {
        assert!(parsed(source, POLICY).is_empty(), "{source}");
    }
}

#[test]
fn reusable_calls_and_nested_steps_are_both_judged() {
    let options = r#"
actions:
  - uses: example/publish-widget
    require:
      keep-days: 1
      replace: true
  - uses: example/ship-items
    ban: [overwrite]
"#;
    let source = r#"
jobs:
  publish:
    uses: example/publish-widget@v1
    with:
      keep-days: 30
      replace: true
    steps:
      - name: ship items
        uses: example/ship-items@v1
        with:
          overwrite: true
"#;
    let findings = parsed(source, options);
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().any(|finding| {
        finding.line == 6
            && finding.target.as_deref() == Some("publish")
            && finding.message.contains("job \"publish\" calls")
            && !finding.message.contains("step")
    }));
    assert!(findings.iter().any(|finding| {
        finding.message.contains("step \"ship items\"") && finding.message.contains("overwrite")
    }));
}

#[test]
fn reusable_input_lines_prefer_the_key_then_the_job() {
    let source = r#"
jobs:
  publish:

    uses: example/publish-widget@v1
  ship:
    runs-on: ubuntu-latest
"#;
    assert_eq!(
        super::super::lines::job_input_line(source, "publish", "keep-days"),
        5
    );
    assert_eq!(
        super::super::lines::job_input_line(source, "ship", "keep-days"),
        6
    );
    assert_eq!(
        super::super::lines::job_input_line(source, "missing", "uses"),
        1
    );
}

#[test]
fn merge_keys_resolve_before_inputs_are_judged() {
    let supplied = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<:
            keep-days: 1
            replace: true
"#;
    let value: Value = serde_yaml::from_str(supplied).unwrap();
    let with = value
        .get("jobs")
        .and_then(|jobs| jobs.get("build"))
        .and_then(|job| job.get("steps"))
        .and_then(Value::as_sequence)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("with"))
        .and_then(Value::as_mapping)
        .unwrap();
    assert!(with.get(Value::String("<<".to_string())).is_some());
    assert!(with.get(Value::String("keep-days".to_string())).is_none());
    assert!(parsed(supplied, POLICY).is_empty());
    let overridden = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<:
            keep-days: 1
            replace: true
          keep-days: 30
"#;
    let findings = parsed(overridden, POLICY);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("30"));
    let sequence = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<:
            - keep-days: 30
              replace: false
            - 1
            - keep-days: 1
              replace: true
"#;
    assert!(parsed(sequence, POLICY).is_empty(), "{sequence}");
    let nested = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<:
            <<:
              keep-days: 1
              replace: true
"#;
    assert!(parsed(nested, POLICY).is_empty());
    let scalar = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<: 1
"#;
    assert!(parsed(scalar, POLICY).iter().any(|finding| finding
        .message
        .contains("without required input `keep-days`")));
    let nulled = r#"
jobs:
  build:
    steps:
      - uses: example/publish-widget@v1
        with:
          <<:
            keep-days: 1
            replace: true
          keep-days:
"#;
    let cleared = parsed(nulled, POLICY);
    assert!(cleared.iter().any(|finding| finding
        .message
        .contains("without required input `keep-days`")));
    assert!(cleared
        .iter()
        .all(|finding| !finding.message.contains("`replace`")));
}

#[test]
fn banned_input_from_a_merge_key_reports() {
    let options = r#"
actions:
  - uses: example/ship-items
    ban: [overwrite]
"#;
    let source = r#"
jobs:
  build:
    steps:
      - uses: example/ship-items@v1
        with:
          <<:
            overwrite: true
"#;
    let findings = parsed(source, options);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("banned input `overwrite`"));
}

#[test]
fn saved_call_fixtures_report_composite_reusable_and_merge_shapes() {
    let composite = run_file("composite", ".github/actions/publish-widget/action.yml");
    assert_eq!(composite.len(), 1, "{composite:?}");
    assert!(composite[0]
        .message
        .contains("composite step \"publish widgets\""));
    let reusable = run_file("reusable", ".github/workflows/widgets.yml");
    assert_eq!(reusable.len(), 2, "{reusable:?}");
    assert!(reusable.iter().any(|finding| {
        finding.message.contains("job \"publish\" calls") && finding.message.contains("30")
    }));
    assert!(reusable.iter().any(|finding| {
        finding.message.contains("job \"pack\" calls")
            && finding
                .message
                .contains("without required input `keep-days`")
    }));
    let merged = run_file("merge-keys", ".github/workflows/widgets.yml");
    assert_eq!(merged.len(), 2, "{merged:?}");
    assert!(merged
        .iter()
        .all(|finding| !finding.message.contains("without required")));
    assert!(merged
        .iter()
        .any(|finding| finding.message.contains("banned input `overwrite`")));
    assert!(merged.iter().any(|finding| {
        finding.message.contains("publish orders") && finding.message.contains("30")
    }));
}
