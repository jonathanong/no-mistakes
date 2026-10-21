use super::*;

#[test]
fn dense_branch_fixture_keeps_query_findings_and_tracks_copied_slots() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-branch-copies")
        .canonicalize()
        .expect("saved branch fixture should exist");
    let (output, diagnostics) = check_json_observed(&root, true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let entry =
        include_str!("../../../../../fixtures/performance/postgres-branch-copies/src/entry-00.mts");
    let unsafe_line = entry
        .lines()
        .position(|line| line.ends_with("// unsafe-query"))
        .unwrap() as u64
        + 1;
    assert_eq!(report["warnings"], json!([]));
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 12);
    assert!(findings.iter().all(|finding| {
        finding["rule"] == "postgres-require-query-annotation"
            && finding["line"].as_u64() == Some(unsafe_line)
            && finding["message"].as_str().is_some_and(|message| {
                message.ends_with("query execution must start with a /* name */ annotation")
            })
    }));
    assert_eq!(diagnostics.work["parse.files"], 14);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 12);
    assert_eq!(
        diagnostics.work["postgres.annotation.incoming_dense_slots_copied"],
        24,
        "twelve executor roots share unchanged dense slots and copy two differing incoming candidates each"
    );
}

#[test]
fn one_opaque_callback_root_preserves_findings_and_exposes_repeated_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-single-root-opaque-arms")
        .canonicalize()
        .unwrap();
    for jobs in [1, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        let baseline = pool.install(|| check_json(&root)).unwrap();
        let (observed, diagnostics) = pool.install(|| check_json_observed(&root, true)).unwrap();
        assert_eq!(observed, baseline, "observer parity, jobs={jobs}");
        let report: serde_json::Value = serde_json::from_str(&observed).unwrap();
        assert_eq!(report["warnings"], json!([]));
        assert_eq!(report["advisories"], json!([]));
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0]["file"], "entry.mts");
        assert_eq!(findings[0]["line"], 12);
        assert_eq!(findings[0]["rule"], "postgres-require-query-annotation");
        assert_eq!(diagnostics.work["parse.files"], 3);
        assert_eq!(diagnostics.work["postgres.annotation.roots"], 1);
        assert_eq!(
            diagnostics.work["postgres.annotation.module_initializations_completed"],
            3
        );
        // A single root still performs many joins/walks. These are work counts,
        // not elapsed-time evidence or a claim that the arms are independent.
        assert_eq!(
            diagnostics.work["postgres.annotation.reachability_walks"],
            19
        );
        assert_eq!(
            diagnostics.work["postgres.annotation.prefix_compare_arm"],
            12
        );
        assert_eq!(diagnostics.work["postgres.annotation.prefix_nodes"], 200);
    }
}
