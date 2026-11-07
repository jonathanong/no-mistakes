use super::*;
#[test]
fn helper_effects_require_matching_executor_access_and_awaited_completion() {
    for name in ["helper-completion", "helper-imported-completion"] {
        let root = fixture(name);
        let path = root.join("src/query.ts");
        let source = std::fs::read_to_string(&path).unwrap();
        let mut files = vec![path];
        if name.contains("imported") {
            files.push(root.join("src/helper.ts"));
        }
        let findings = check_with_files(&root, &config(), &files).unwrap();
        assert_eq!(
            source
                .lines()
                .filter(|line| line.contains("FOR UPDATE"))
                .count(),
            9
        );
        let mut caller = "";
        for (index, line) in source.lines().enumerate() {
            if line.starts_with("export async function caller") {
                caller = line;
            }
            if !line.contains("FOR UPDATE") {
                continue;
            }
            let diagnostics: Vec<_> = findings
                .iter()
                .filter(|finding| finding.file == "src/query.ts" && finding.line == index + 1)
                .collect();
            if caller.contains("Valid") {
                assert!(diagnostics.is_empty(), "{name} {caller}: {findings:?}");
            } else {
                assert!(
                    diagnostics
                        .iter()
                        .any(|finding| finding.target.as_deref() == Some("unanalyzable")),
                    "{name} {caller}: {findings:?}"
                );
                assert!(
                    !diagnostics
                        .iter()
                        .any(|finding| finding.target.as_deref() == Some(CHECKS[0])),
                    "{name} {caller}: {findings:?}"
                );
            }
        }
        assert!(
            !findings
                .iter()
                .any(|finding| finding.target.as_deref() == Some(CHECKS[3])),
            "{name}: {findings:?}"
        );
    }
}
