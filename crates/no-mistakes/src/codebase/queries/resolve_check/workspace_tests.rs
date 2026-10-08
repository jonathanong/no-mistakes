use super::tests::named_fixture;
use super::*;
use crate::codebase::ts_resolver::ImportResolver;
use std::path::Path;

#[test]
fn workspace_imports_resolve_and_missing_subpaths_fail_closed() {
    for tsconfig in [None, Some(PathBuf::from("tsconfig.json"))] {
        let args = ResolveCheckArgs {
            files: vec![PathBuf::from("packages/app/entry.mts")],
            root: Some(named_fixture("workspace-resolve-check")),
            tsconfig,
            format: None,
            json: false,
        };
        let report = compute(&args).unwrap();
        assert!(!report.all_resolve);
        let rows = report
            .imports
            .iter()
            .map(|row| {
                (
                    row.specifier.as_str(),
                    row.kind,
                    row.status,
                    row.resolved.as_deref(),
                    row.computed,
                )
            })
            .collect::<Vec<_>>();
        let expected = [
            (
                "@fx/lib/target",
                "static",
                Status::Resolved,
                Some("packages/lib/target.mts"),
                false,
            ),
            ("@fx/lib/missing", "static", Status::Unresolved, None, false),
            ("@fx/not-a-package", "static", Status::External, None, false),
            (
                "@fx/lib/missing-dynamic",
                "dynamic",
                Status::Unresolved,
                None,
                false,
            ),
            (
                "@fx/lib/types.d",
                "type",
                Status::Resolved,
                Some("packages/lib/types.d.mts"),
                false,
            ),
            ("@fx/lib/types.d", "static", Status::Unresolved, None, false),
            (
                "@fx/main",
                "static",
                Status::Resolved,
                Some("packages/main/entry.mts"),
                false,
            ),
            (
                "@fx/closed/private",
                "static",
                Status::Unresolved,
                None,
                false,
            ),
            (
                "@fx/lib/target",
                "require",
                Status::Resolved,
                Some("packages/lib/target.mts"),
                false,
            ),
            (
                "@fx/lib/target",
                "require-resolve",
                Status::Resolved,
                Some("packages/lib/target.mts"),
                false,
            ),
            ("third-party", "static", Status::External, None, false),
            ("node:fs", "static", Status::External, None, false),
            (
                "./relative.mjs",
                "static",
                Status::Resolved,
                Some("packages/app/relative.mts"),
                false,
            ),
            ("./missing.mjs", "static", Status::Unresolved, None, false),
            (
                "@fx/lib/alias",
                "static",
                Status::Resolved,
                Some("packages/app/alias.mts"),
                false,
            ),
            (
                "@fx/lib/alias-missing",
                "static",
                Status::Unresolved,
                None,
                false,
            ),
            (
                "#local",
                "static",
                Status::Resolved,
                Some("packages/app/relative.mts"),
                false,
            ),
            ("#missing", "static", Status::Unresolved, None, false),
            ("name", "dynamic", Status::Unresolved, None, true),
        ];
        assert_eq!(rows.len(), expected.len());
        for (actual, expected) in rows.iter().zip(expected) {
            assert!(actual == &expected, "unexpected row for {}", actual.0);
        }
        let batch: serde_json::Value =
            serde_json::from_str(&run_json_batch(args).unwrap()).unwrap();
        assert_eq!(batch["allResolve"], false);
        assert_eq!(
            batch["unresolvedFiles"],
            serde_json::json!(["packages/app/entry.mts"])
        );
    }
}

#[test]
fn workspace_resolve_check_parses_only_requested_files() {
    let root = named_fixture("workspace-resolve-check");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let reports = compute_many(&ResolveCheckArgs {
        files: vec![PathBuf::from("packages/app/valid.mts")],
        root: Some(root.clone()),
        tsconfig: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert!(reports[0].all_resolve);
    let work = observer.snapshot().work;
    assert_eq!(work["ts_facts.collections"], 1);
    assert_eq!(work["ts_facts.files"], 1);
    assert_eq!(
        work["parse.requests"], 1,
        "workspace metadata must not trigger another source fact pass"
    );
    let reads = observer.source_read_snapshot();
    assert!(reads.values().all(|count| *count == 1), "{reads:#?}");
    assert_eq!(observer.snapshot().work["workspace.builds"], 1);
}

#[test]
fn workspace_target_outside_prepared_visibility_is_unresolved() {
    let root = named_fixture("workspace-resolve-check");
    let target = super::super::shared::resolve_target(
        Path::new("packages/app/valid.mts"),
        Some(&root),
        None,
    )
    .unwrap();
    let mut visible = target.visible_files().clone();
    visible.remove(&root.join("packages/lib/target.mts"));
    let resolver =
        ImportResolver::new_in_session(target.tsconfig().unwrap(), Some(&visible), &target.session);
    let facts =
        super::super::reverse::collect_target_import_facts(&target, std::slice::from_ref(&target));
    let import = facts[&target.abs_file]
        .imports
        .iter()
        .find(|imp| imp.specifier == "@fx/lib/target")
        .unwrap();
    let row = classify_prepared(
        import,
        &target.abs_file,
        &root,
        &resolver,
        &target.session.workspace(&root),
        &visible,
    );
    assert!(row.status == Status::Unresolved);
    assert!(row.resolved.is_none());
}
