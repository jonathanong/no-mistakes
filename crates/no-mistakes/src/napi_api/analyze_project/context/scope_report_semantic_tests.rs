#[test]
fn p6_effects_reports_preserve_category_selection_and_depth() {
    let (_fixture, root) = p6_analysis_fixture("effects");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":root,
        "reports":[
            {"type":"effects", "kind":"valkey", "entry":"app/server.ts"},
            {"type":"effects", "kind":"valkey", "entry":"app/server.ts", "categories":["cache"]},
            {"type":"effects", "kind":"valkey", "entry":"app/server.ts", "depth":0}
        ]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let reports = prepared
        .reports
        .iter()
        .map(|request| {
            serde_json::from_str::<Value>(context.effects_report(request, &prepared).unwrap().get())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(reports[0]["kind"], "valkey");
    assert_eq!(reports[0]["entry"], "app/server.ts");
    assert!(reports[0]["byCategory"]["pubsub"].as_u64().unwrap() > 0);
    assert_eq!(reports[1]["callSites"].as_array().unwrap().len(), 2);
    assert!(reports[1]["callSites"]
        .as_array()
        .unwrap()
        .iter()
        .all(|site| {
            site["file"] == "lib/cache.ts" && site["category"] == "cache" && site["depth"] == 1
        }));
    assert!(reports[2]["callSites"].as_array().unwrap().is_empty());
}

#[test]
fn p6_rsc_reports_preserve_depth_and_stop_at_client_boundaries() {
    let (_fixture, root) = p6_analysis_fixture("rsc-callers");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":root,
        "reports":[
            {"type":"rscCallers", "component":"app/ui/Button.tsx"},
            {"type":"rscCallers", "component":"app/ui/Button.tsx", "depth":1}
        ]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let reports = prepared
        .reports
        .iter()
        .map(|request| {
            serde_json::from_str::<Value>(
                context
                    .rsc_callers_report(request, &prepared)
                    .unwrap()
                    .get(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(reports[0]["component"], "app/ui/Button.tsx");
    let callers = reports[0]["callers"].as_array().unwrap();
    assert!(callers
        .iter()
        .any(|caller| caller["file"] == "app/dashboard/page.tsx"
            && caller["kind"] == "page"
            && caller["depth"] == 2));
    assert!(callers
        .iter()
        .any(|caller| caller["file"] == "app/ui/ServerWidget.tsx"
            && caller["environment"] == "server"));
    assert!(!callers.iter().any(|caller| matches!(
        caller["file"].as_str(),
        Some("app/ui/ClientThing.tsx" | "app/ui/ClientParent.tsx")
    )));
    let shallow = reports[1]["callers"].as_array().unwrap();
    assert_eq!(shallow.len(), 2);
    assert!(shallow.iter().all(|caller| caller["depth"] == 1));
}

#[test]
fn p6_prepared_call_sites_success_returns_the_config_callers() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/queries/call-sites-ordinary-scope");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":fixture.path(),
        "reports":[{"type":"callSites", "file":"packages/unit/src/subject.ts", "exportName":"subject"}]
    })).unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let report = context
        .command_report(&prepared.reports[0], &prepared)
        .unwrap();
    let report: Value = serde_json::from_str(report.get()).unwrap();
    let mut callers = report["callSites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    callers.sort();
    assert_eq!(
        callers,
        ["packages/unit/vitest.config.ts", "playwright.config.ts"]
    );
    let mut missing_export = prepared.reports[0].clone();
    missing_export
        .options
        .insert("exportName".into(), json!("p6Missing"));
    let error = context
        .command_report(&missing_export, &prepared)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("`p6Missing` is not a value export"),
        "{error:#}"
    );
}
