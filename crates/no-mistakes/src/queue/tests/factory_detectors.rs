use super::*;
use std::path::Path;

#[test]
fn detector_gaps_report_renamed_jobs_member_waits_and_wrapper_options() {
    let root = fixture("factory-detectors");
    let report = analyze_project(&root, None, &[]).unwrap();
    let count = |findings: &[crate::queue::graph_model::CheckFinding], suffix: &str, kind: &str| {
        findings
            .iter()
            .filter(|finding| finding.file.ends_with(suffix) && finding.kind == kind)
            .count()
    };
    assert!(
        count(
            &report.check,
            "workers/renamed-param.mts",
            "self-dedup-enqueue"
        ) >= 3,
        "renamed, destructured, and later `job` params should dedup, check: {:?}",
        report.check
    );
    assert_eq!(
        count(
            &report.check,
            "enqueues/wrapper-dedup.mts",
            "self-dedup-enqueue"
        ),
        1,
        "wrapper options should dedup, check: {:?}",
        report.check
    );
    assert!(
        count(&report.check, "enqueues/wrapper-dedup.mts", "debounce-ttl") >= 1,
        "wrapper options should see debounce ttl, check: {:?}",
        report.check
    );
    assert_eq!(
        count(&report.check, "routes/member-wait.mts", "enqueue-and-wait"),
        2,
        "member and identifier waits should both be reported, check: {:?}",
        report.check
    );
    assert_eq!(
        count(&report.check, "routes/next-line.mts", "enqueue-and-wait"),
        2
    );
    assert_eq!(
        count(
            &report.check,
            "workers/ignored-wait.mts",
            "enqueue-and-wait"
        ),
        0,
        "waits outside waitScopes are not findings, check: {:?}",
        report.check
    );
    assert!(count(&report.check, "workers/disabled.mts", "self-dedup-enqueue") >= 1);
    assert_eq!(
        count(
            &report.check,
            "workers/nested-callback.mts",
            "self-dedup-enqueue"
        ),
        1,
        "only a nested parameter named `job` dedups, check: {:?}",
        report.check
    );
    for job in ["p1", "p2", "p3", "p5", "p8"] {
        assert!(
            report.check.iter().any(|finding| {
                finding.file.ends_with("workers/wrapped-processor.mts")
                    && finding.kind == "self-dedup-enqueue"
                    && finding.job.as_deref() == Some(job)
            }),
            "{job} should dedup through a wrapped processor, check: {:?}",
            report.check
        );
    }

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
    assert_eq!(
        count(&findings, "workers/disabled.mts", "self-dedup-enqueue"),
        0
    );
    assert_eq!(
        count(&findings, "routes/next-line.mts", "enqueue-and-wait"),
        1
    );
    assert_eq!(
        findings
            .iter()
            .find(|finding| finding.file.ends_with("routes/next-line.mts"))
            .unwrap()
            .line,
        6
    );
    assert!(count(&findings, "workers/renamed-param.mts", "self-dedup-enqueue") >= 3);
}

#[test]
fn conflicting_factory_indexes_are_rejected_and_wait_scopes_narrow() {
    let queue_yaml = "version: 2\nqueues:\n  factories:\n    queue:\n      - { callee: createQueue, name: 0 }\n      - { callee: createQueue, name: 1 }\n";
    let queue_err = serde_yaml::from_str::<crate::config::v2::NoMistakesConfig>(queue_yaml)
        .unwrap_err()
        .to_string();
    assert!(queue_err.contains("conflicting queue"), "{queue_err}");

    let enqueue_yaml = "version: 2\nqueues:\n  factories:\n    enqueue:\n      - { callee: enqueueJob, queue: 0, job: 1 }\n      - { callee: enqueueJob, queue: 0, job: 2 }\n";
    let enqueue_err = serde_yaml::from_str::<crate::config::v2::NoMistakesConfig>(enqueue_yaml)
        .unwrap_err()
        .to_string();
    assert!(enqueue_err.contains("conflicting enqueue"), "{enqueue_err}");

    let worker_yaml = "version: 2\nqueues:\n  factories:\n    worker:\n      - { callee: createWorker, name: 0, processor: 1 }\n      - { callee: createWorker, name: 1, processor: 1 }\n";
    let worker_err = serde_yaml::from_str::<crate::config::v2::NoMistakesConfig>(worker_yaml)
        .unwrap_err()
        .to_string();
    assert!(worker_err.contains("conflicting worker"), "{worker_err}");

    let ok_yaml = "version: 2\nprojects:\n  jobs:\n    root: services\n    queues:\n      waitScopes: [\"workers/**\", \"services/already/**\", \"services\"]\n  web:\n    root: apps/web\nqueues:\n  factories:\n    enqueue:\n      - { callee: enqueueJob, queue: 0, job: 1, options: 3 }\n";
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(ok_yaml).unwrap();
    let roles = config
        .queues
        .factories
        .clone()
        .with_project_wait_scopes(&config);
    let jobs = roles
        .wait_scopes
        .iter()
        .find(|project| project.root == "services")
        .unwrap();
    assert_eq!(
        jobs.scopes,
        vec![
            "services".to_string(),
            "services/already/**".to_string(),
            "services/workers/**".to_string(),
        ]
    );
    let web = roles
        .wait_scopes
        .iter()
        .find(|project| project.root == "apps/web")
        .unwrap();
    assert!(web.scopes.is_empty());
    let repo = Path::new("/repo");
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("/repo/apps/web/page.mts"),
        repo,
        &roles.wait_scopes
    ));
    assert!(!crate::queue::extract::waits_in_scope(
        Path::new("/repo/services/routes/a.mts"),
        repo,
        &roles.wait_scopes
    ));
    assert!(!crate::queue::extract::waits_in_scope(
        Path::new("/repo/./services/../services/routes/a.mts"),
        Path::new("/repo/"),
        &roles.wait_scopes
    ));
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("/repo/services/workers/a.mts"),
        repo,
        &roles.wait_scopes
    ));
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("/repo/other/a.mts"),
        repo,
        &roles.wait_scopes
    ));
    let mut other = roles.clone();
    other.enqueue[0].job = 9;
    let mut merged = roles.clone();
    assert!(merged.merge(&other).is_err());
    assert_eq!(merged.enqueue[0].job, 1);
    other.enqueue[0].job = 1;
    other.worker.push(crate::config::v2::schema::WorkerFactory {
        callee: "createWorker".to_string(),
        name: 0,
        processor: 2,
    });
    let mut worker_roles = roles.clone();
    worker_roles
        .worker
        .push(crate::config::v2::schema::WorkerFactory {
            callee: "createWorker".to_string(),
            name: 0,
            processor: 1,
        });
    assert!(worker_roles.merge(&other).is_err());
    let mut queue_roles = roles.clone();
    queue_roles
        .queue
        .push(crate::config::v2::schema::QueueNameFactory {
            callee: "createQueue".to_string(),
            name: 0,
        });
    other
        .queue
        .push(crate::config::v2::schema::QueueNameFactory {
            callee: "createQueue".to_string(),
            name: 1,
        });
    assert!(queue_roles.merge(&other).is_err());

    let routes = crate::config::v2::schema::ProjectWaitScopes {
        root: String::new(),
        scopes: vec!["routes/**".to_string()],
    };
    let broken = crate::config::v2::schema::ProjectWaitScopes {
        root: String::new(),
        scopes: vec!["[".to_string()],
    };
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("routes/a.mts"),
        repo,
        &[]
    ));
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("/repo/routes/a.mts"),
        repo,
        std::slice::from_ref(&routes)
    ));
    assert!(!crate::queue::extract::waits_in_scope(
        Path::new("/repo/workers/a.mts"),
        repo,
        std::slice::from_ref(&routes)
    ));
    assert!(crate::queue::extract::waits_in_scope(
        Path::new("anywhere.mts"),
        repo,
        std::slice::from_ref(&broken)
    ));
}
