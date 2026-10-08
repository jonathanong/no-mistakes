use super::common::{file_paths, fixture, run_json};

#[test]
fn import_dynamic_follows_if_switch_nested_function_and_jsx_handler() {
    let root = fixture("import-forms");
    let value = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "import-dynamic",
            "dynamic-all-shapes.tsx",
        ],
    );
    let mut paths = file_paths(&value);
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "dynamic-click-target.mts",
            "dynamic-if-target.mts",
            "dynamic-nested-target.mts",
            "dynamic-switch-target.mts",
        ],
        "{value:#?}"
    );
}

#[test]
fn conditional_workspace_imports_follow_literal_exports_and_package_imports() {
    let root = fixture("conditional-workspace-imports");
    let expected = vec![
        "packages/app/internal/target.mts",
        "packages/app/relative.mts",
        "packages/lib/target-a.mts",
        "packages/lib/target-b.mts",
    ];
    for file in [
        "conditional.mts",
        "conditional.test.mts",
        "sequential.mts",
        "top-level.mts",
    ] {
        for relationships in [
            vec!["import"],
            vec!["import-dynamic"],
            vec!["import", "workspace"],
            vec!["import-dynamic", "workspace"],
        ] {
            let file = format!("packages/app/{file}");
            let mut args = vec!["dependencies", &file];
            for relationship in &relationships {
                args.extend(["--relationship", relationship]);
            }
            let report = run_json(&root, &args);
            assert_eq!(
                file_paths(&report),
                expected,
                "{file}: {relationships:?}: {report:#?}"
            );
            let dynamic_kind = if file.ends_with("top-level.mts") {
                "dynamic-import"
            } else {
                "conditional-dynamic-import"
            };
            assert!(
                super::common::via_kinds(&report, "packages/lib/target-a.mts")
                    .contains(&dynamic_kind.to_string())
            );
        }
    }
    assert_eq!(
        file_paths(&run_json(
            &root,
            &[
                "dependencies",
                "packages/app/static.mts",
                "--relationship",
                "import",
                "--relationship",
                "workspace"
            ]
        )),
        expected
    );
    let workspace = run_json(
        &root,
        &[
            "dependencies",
            "packages/app/conditional.mts",
            "--relationship",
            "workspace",
        ],
    );
    assert_eq!(
        file_paths(&workspace),
        vec![
            "packages/app/internal/target.mts",
            "packages/lib/target-a.mts",
            "packages/lib/target-b.mts"
        ]
    );
    for command in ["dependents", "related"] {
        let report = run_json(
            &root,
            &[
                command,
                "packages/lib/target-a.mts",
                "--relationship",
                "import-dynamic",
            ],
        );
        assert_eq!(
            file_paths(&report),
            vec![
                "packages/app/conditional.mts",
                "packages/app/conditional.test.mts",
                "packages/app/sequential.mts",
                "packages/app/top-level.mts"
            ],
            "{command}: {report:#?}"
        );
    }
    assert!(file_paths(&run_json(
        &root,
        &[
            "dependencies",
            "packages/app/computed.mts",
            "--relationship",
            "import-dynamic",
            "--relationship",
            "workspace"
        ]
    ))
    .is_empty());
}

#[test]
fn computed_workspace_imports_remain_unresolved() {
    let root = fixture("conditional-workspace-imports");
    let output = super::common::run(&[
        "resolve-check",
        "packages/app/computed.mts",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["allResolve"], false);
    let imports = report["imports"].as_array().unwrap();
    assert_eq!(imports.len(), 2);
    assert!(imports
        .iter()
        .all(|import| import["computed"] == true && import["status"] == "unresolved"));
}

#[test]
fn conditional_workspace_import_filters_keep_type_edges_separate() {
    let root = fixture("conditional-workspace-imports");
    assert!(file_paths(&run_json(
        &root,
        &[
            "dependencies",
            "packages/app/type-only.mts",
            "--relationship",
            "import-dynamic"
        ]
    ))
    .is_empty());
    assert_eq!(
        file_paths(&run_json(
            &root,
            &[
                "dependencies",
                "packages/app/type-only.mts",
                "--relationship",
                "workspace"
            ]
        )),
        vec!["packages/lib/type-target.mts"]
    );
}

#[test]
fn conditional_workspace_imports_explicit_config_compact_depth_matches_reproduction() {
    let root = fixture("conditional-workspace-imports");
    for relationships in [
        vec!["import-dynamic"],
        vec!["import-dynamic", "workspace"],
        vec!["import", "workspace"],
    ] {
        let mut args = vec![
            "dependencies",
            "packages/app/conditional.test.mts",
            "--tsconfig",
            "tsconfig.json",
            "--depth",
            "10",
            "--projection",
            "paths",
        ];
        for relationship in &relationships {
            args.extend(["--relationship", relationship]);
        }
        let report = run_json(&root, &args);
        assert_eq!(
            report["files"],
            serde_json::json!([
                "packages/app/internal/target.mts",
                "packages/app/relative.mts",
                "packages/lib/target-a.mts",
                "packages/lib/target-b.mts"
            ])
        );
    }
}
