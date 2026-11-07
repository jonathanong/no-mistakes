use super::*;

#[test]
fn finite_groups_and_batch_reads_preserve_open_ended_controls() {
    let fixtures = root().parent().unwrap().join("bounded-shapes");
    for (shape, count) in [
        ("counter", 20),
        ("tuple-enum", 29),
        ("chunk", 16),
        ("keyset", 46),
        ("closed-keys", 22),
        ("distinct-calls", 1),
        ("dependent-chain", 1),
    ] {
        let root = fixtures.join(shape);
        let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
        assert_eq!(findings.len(), count, "{shape}: {findings:#?}");
        assert!(
            findings.iter().all(|finding| {
                finding.target.as_deref().is_some_and(|target| {
                    target.starts_with("postgres: control")
                        || target.starts_with("postgres: nested")
                })
            }),
            "{shape}: {findings:#?}"
        );
        assert!(
            crate::codebase::rules::run_check(&root, Some(&root.join("off.yml")), None)
                .unwrap()
                .is_empty()
        );
    }
}
