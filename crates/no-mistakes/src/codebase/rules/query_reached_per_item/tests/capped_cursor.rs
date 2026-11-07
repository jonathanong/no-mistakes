use super::*;

#[test]
fn cursor_option_removes_only_proven_page_reads_and_defaults_to_false() {
    let root = root().parent().unwrap().join("capped-cursor");
    let run = |config: &str| {
        crate::codebase::rules::run_check(&root, Some(&root.join(config)), None)
            .unwrap()
            .into_iter()
            .filter(|finding| finding.rule == RULE_ID)
            .collect::<Vec<_>>()
    };
    let default = run(".no-mistakes.yml");
    let off = run("off.yml");
    let on = run("on.yml");
    assert_eq!(default, off);
    let removed = off
        .iter()
        .filter(|finding| !on.contains(finding))
        .collect::<Vec<_>>();
    assert_eq!(
        removed
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        [
            "src/conditional_spread.ts",
            "src/helper_loop.ts",
            "src/proven.ts",
        ]
    );
    assert!(on.iter().all(|finding| off.contains(finding)));
    assert!(on
        .iter()
        .any(|finding| finding.file == "src/proven.ts"
            && finding.import.as_deref() == Some("handle")));
    assert!(on
        .iter()
        .any(|finding| finding.file == "src/page-read.ts"
            && finding.import.as_deref() == Some("read")));
}
