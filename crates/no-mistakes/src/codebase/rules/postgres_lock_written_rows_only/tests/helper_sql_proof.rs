use super::*;
#[test]
fn helper_sql_requires_untagged_interpolation_free_literals() {
    for name in ["helper-sql-proof", "helper-imported-sql-proof"] {
        let root = fixture(name);
        let path = root.join("src/query.ts");
        let source = std::fs::read_to_string(&path).unwrap();
        let mut files = vec![path];
        if name.contains("imported") {
            files.push(root.join("src/helper.ts"));
        }
        let findings = check_with_files(&root, &config(), &files).unwrap();
        let mut caller = "";
        let mut checked = 0;
        for (index, line) in source.lines().enumerate() {
            if line.starts_with("export async function caller") {
                caller = line;
            }
            let valid = caller.contains("stringWrite")
                || caller.contains("templateWrite")
                || caller.contains("literalLock");
            let lock_helper = caller.contains("Lock");
            let target = if lock_helper {
                line.contains("await interpolatedLock(")
                    || line.contains("await taggedLock(")
                    || line.contains("await literalLock(")
            } else {
                line.contains("await tx.query(`SELECT 1 FROM users")
            };
            if !target {
                continue;
            }
            checked += 1;
            let diagnostics: Vec<_> = findings
                .iter()
                .filter(|finding| finding.file == "src/query.ts" && finding.line == index + 1)
                .collect();
            if valid {
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
        assert_eq!(checked, 9);
    }
}
