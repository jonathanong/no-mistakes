use super::*;
use crate::config::v2::schema::{CheckCommandDef, CheckFileArgs};

#[test]
fn generic_only_repository_skips_test_file_discovery_and_graph() {
    let mut args = args(&["src/value.ts"]);
    args.root = generic_only_fixture();
    args.tsconfig = Some(args.root.join("missing-tsconfig.json"));
    let (report, stats) = generate_impacted_checks_with_stats(&args).unwrap();
    assert_eq!(command_strings(&report), ["eslint src/value.ts"]);
    assert_eq!((stats.framework_discoveries, stats.graph_builds), (0, 0));
}
#[test]
fn explicit_generic_only_skips_configured_frameworks_and_keeps_changed_files() {
    let mut args = args(&["src/value.ts"]);
    args.root = multi_framework_fixture();
    args.config = Some(generic_only_config());
    args.generic_only = true;
    let mut timing = timing::TimingTracker::new(false, true);
    let (report, stats) =
        super::super::generate_impacted_checks_with_timing(&args, &mut timing).unwrap();
    timing.finish_total();
    assert_eq!(
        command_strings(&report),
        ["echo always", "pnpm exec eslint"]
    );
    assert!(report
        .checks
        .iter()
        .all(|check| check.kind == CheckKind::Generic));
    assert_eq!(report.changed_files, ["src/value.ts"]);
    assert!(report.warnings.is_empty() && !report.fallback_triggered);
    assert_eq!((stats.framework_discoveries, stats.graph_builds), (0, 0));
    assert_eq!(
        timing
            .into_timings()
            .unwrap()
            .into_iter()
            .map(|item| item.phase)
            .collect::<Vec<_>>(),
        ["prepare", "generic-checks", "total"]
    );
}
#[test]
fn generic_checks_excludes_deleted_from_append() {
    let mut config = NoMistakesConfig::default();
    config.checks.commands = vec![
        CheckCommandDef {
            name: "eslint".into(),
            include: vec!["**/*.ts".into()],
            command: vec!["eslint".into()],
            file_args: CheckFileArgs::Append,
            ..Default::default()
        },
        CheckCommandDef {
            name: "only-deleted".into(),
            include: vec!["gone/**".into()],
            command: vec!["lint".into()],
            file_args: CheckFileArgs::Append,
            ..Default::default()
        },
        CheckCommandDef {
            name: "tsc".into(),
            include: vec!["**/*.ts".into()],
            command: vec!["tsc".into()],
            file_args: CheckFileArgs::None,
            ..Default::default()
        },
    ];
    let changed = vec!["a.ts".into(), "b.ts".into(), "gone/x.ts".into()];
    let deleted = ["a.ts".into(), "gone/x.ts".into()].into_iter().collect();
    let checks = generic_checks(&config, &changed, &deleted).unwrap();
    assert_eq!(
        checks
            .iter()
            .find(|check| check.name == "eslint")
            .unwrap()
            .command,
        ["eslint", "b.ts"]
    );
    assert!(!checks.iter().any(|check| check.name == "only-deleted"));
    assert!(checks.iter().any(|check| check.name == "tsc"));
}
#[test]
fn generic_checks_normalizes_dot_slash_globs() {
    let mut config = NoMistakesConfig::default();
    config.checks.commands = vec![CheckCommandDef {
        name: "eslint".into(),
        include: vec!["./src/**/*.ts".into()],
        command: vec!["eslint".into()],
        file_args: CheckFileArgs::Append,
        ..Default::default()
    }];
    let checks = generic_checks(&config, &["src/foo.ts".into()], &BTreeSet::new()).unwrap();
    assert_eq!(checks[0].command, ["eslint", "src/foo.ts"]);
}
#[test]
fn generic_checks_always_emits_for_empty_changes_and_keeps_normalized_files() {
    let mut config = NoMistakesConfig::default();
    config.checks.commands = vec![CheckCommandDef {
        name: "always".into(),
        command: vec!["echo".into(), "always".into()],
        file_args: CheckFileArgs::None,
        always: true,
        ..Default::default()
    }];
    let changed = vec!["src/a.ts".into(), "src/b.ts".into()];
    let checks = generic_checks(&config, &changed, &BTreeSet::new()).unwrap();
    assert_eq!(checks[0].files, changed);
    let empty = generic_checks(&config, &[], &BTreeSet::new()).unwrap();
    assert_eq!(empty.len(), 1);
    assert!(empty[0].files.is_empty());
}
