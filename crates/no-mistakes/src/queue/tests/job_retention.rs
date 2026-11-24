use super::{analyze_project, fixture};

#[test]
fn queue_retention_reports_each_proven_unbounded_key_once() {
    let report = analyze_project(&fixture("unbounded-job-retention"), None, &[]).unwrap();
    let mut rows = Vec::new();
    for finding in &report.check {
        if finding.kind == "unbounded-job-retention" {
            rows.push((
                finding.file.as_str(),
                finding.queue_name.as_deref(),
                finding.job.as_deref(),
                finding.message.as_str(),
            ));
        }
    }
    rows.sort_unstable();
    assert_eq!(
        rows,
        [
            ("produce.ts", Some("blank"), Some("flow-missing"), BOTH),
            ("produce.ts", Some("blank"), Some("missing-both"), BOTH),
            (
                "produce.ts",
                Some("bounded"),
                Some("bulk-retained"),
                COMPLETE
            ),
            (
                "produce.ts",
                Some("bounded"),
                Some("closed-unbounded-object"),
                FAIL
            ),
            (
                "produce.ts",
                Some("bounded"),
                Some("configured-literal"),
                FAIL
            ),
            ("produce.ts", Some("bounded"), Some("false-last"), COMPLETE),
            (
                "produce.ts",
                Some("bounded"),
                Some("flow-literal"),
                COMPLETE
            ),
            (
                "produce.ts",
                Some("bounded"),
                Some("literal-override"),
                COMPLETE
            ),
            (
                "produce.ts",
                Some("bounded"),
                Some("scheduler-retained"),
                FAIL
            ),
            ("produce.ts", Some("bounded"), Some("suppressed"), FAIL),
            ("produce.ts", Some("bounded"), Some("unbounded-last"), FAIL),
            (
                "produce.ts",
                Some("bounded"),
                Some("unbounded-retention-spread"),
                FAIL
            ),
            ("produce.ts", Some("partial"), Some("missing-fail"), FAIL),
            (
                "produce.ts",
                Some("shared"),
                Some("unbounded-same-name"),
                FAIL
            ),
            ("queues.ts", None, None, FAIL),
            (
                "queues.ts",
                Some("duplicate"),
                Some("conflicting-defaults"),
                FAIL
            ),
            ("queues.ts", Some("factory"), None, COMPLETE),
            ("queues.ts", Some("retained"), None, FAIL),
        ]
    );
    let required_jobs = report
        .check
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .map(|finding| finding.job.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(required_jobs, [Some("allowed-missing")]);
}

const BOTH: &str =
    "job retention is unbounded for removeOnComplete, removeOnFail. Set a count or age limit.";
const COMPLETE: &str = "job retention is unbounded for removeOnComplete. Set a count or age limit.";
const FAIL: &str = "job retention is unbounded for removeOnFail. Set a count or age limit.";
