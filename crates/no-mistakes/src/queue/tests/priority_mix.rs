use super::*;
use crate::queue::graph_model::CheckFinding;

fn mixed(report: &ProjectReport) -> Vec<&CheckFinding> {
    report
        .check
        .iter()
        .filter(|finding| finding.kind == "mixed-priority")
        .collect()
}

fn queue<'a>(findings: &[&'a CheckFinding], name: &str) -> Vec<&'a CheckFinding> {
    findings
        .iter()
        .copied()
        .filter(|finding| finding.queue_name.as_deref() == Some(name))
        .collect()
}

#[test]
fn mixed_literal_priorities_report_the_minority_form() {
    let root = fixture("priority-mix");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report
            .check
            .iter()
            .all(|finding| finding.kind == "mixed-priority"),
        "{:?}",
        report.check
    );
    let findings = mixed(&report);
    assert_eq!(findings.len(), 13, "{findings:?}");

    let widgets = queue(&findings, "widgets");
    assert_eq!(widgets.len(), 1);
    assert_eq!(widgets[0].file, "mixed.ts");
    assert_eq!(widgets[0].line, 4);
    assert_eq!(widgets[0].job.as_deref(), Some("packWidget"));
    assert_eq!(widgets[0].queue_file.as_deref(), Some("queues.ts"));
    assert_eq!(
        widgets[0].message,
        "queue \"widgets\" mixes priority 0 and priority > 0; this producer enqueues with priority 0 while other producers set a priority greater than 0: mixed.ts:8 (rushWidget, priority 5), mixed.ts:12 (fastWidget, priority 2)"
    );

    assert!(queue(&findings, "items").is_empty());
    assert!(queue(&findings, "orders").is_empty());
    let holds = queue(&findings, "holds");
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].job.as_deref(), Some("freeHold"));
    assert!(holds[0].message.contains("paidHold"));
    assert!(holds[0].message.contains("firmHold"));

    for name in [
        "shipments",
        "weights",
        "credits",
        "extras",
        "drafts",
        "tags",
    ] {
        assert!(queue(&findings, name).is_empty(), "{name} should abstain");
    }

    let batches = queue(&findings, "batches");
    assert_eq!(batches.len(), 2);
    assert!(batches.iter().any(|finding| {
        finding.line == 6
            && finding.message
                == "queue \"batches\" mixes priority 0 and priority > 0; this producer enqueues with priority 0 while other producers set a priority greater than 0: batches.ts:10 (countBatch, priority 2)"
    }));
    assert!(batches.iter().any(|finding| {
        finding.line == 10
            && finding.message
                == "queue \"batches\" mixes priority 0 and priority > 0; this producer sets priority 2 while other producers enqueue with priority 0: batches.ts:6 (countBatch)"
    }));

    let parcels = queue(&findings, "parcels");
    assert_eq!(parcels.len(), 1);
    assert_eq!(parcels[0].job.as_deref(), Some("rushParcel"));
    assert!(parcels[0].message.contains("sets priority 7"));
    assert!(parcels[0].message.contains("storeParcel"));
    assert!(parcels[0].message.contains("holdParcel"));

    let routes = queue(&findings, "routes");
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].file, "routes.ts");
    assert_eq!(routes[0].queue_file.as_deref(), Some("routes.ts"));
    assert_eq!(routes[0].job.as_deref(), Some("planRoute"));
    assert!(routes[0].message.contains("rushRoute"));
    assert!(routes[0].message.contains("fastRoute"));

    let catalog = queue(&findings, "catalog");
    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog[0].job.as_deref(), Some("listItem"));
    assert!(catalog[0].message.contains("rushItem"));
    assert!(catalog[0].message.contains("fastItem"));

    assert_eq!(queue(&findings, "labels").len(), 2);
    assert_eq!(queue(&findings, "notes").len(), 2);

    let later = queue(&findings, "later");
    assert_eq!(later.len(), 2);
    assert!(later.iter().any(|finding| {
        finding.job.as_deref() == Some("holdLater") && finding.message.contains("sets priority 4")
    }));
    for name in ["shadows", "spreads", "opaque"] {
        assert!(queue(&findings, name).is_empty(), "{name} should abstain");
    }
}

#[test]
fn mixed_priority_suppression_uses_queues_check_directives() {
    let root = fixture("priority-mix");
    let report = analyze_project(&root, None, &[]).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut findings = report.check;
    crate::codebase::rules::suppress_domain_findings_with_sources(
        &root,
        &mut findings,
        &sources,
        |finding| crate::codebase::rules::SuppressionTarget {
            domain: "queues",
            rule: "queues-check",
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    );
    let labels: Vec<_> = findings
        .iter()
        .filter(|finding| finding.queue_name.as_deref() == Some("labels"))
        .collect();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].job.as_deref(), Some("rushLabel"));
    assert!(findings
        .iter()
        .all(|finding| finding.queue_name.as_deref() != Some("notes")));
    assert!(findings
        .iter()
        .any(|finding| finding.queue_name.as_deref() == Some("widgets")));
}
