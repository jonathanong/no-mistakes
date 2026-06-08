use crate::codebase::symbols::{FileEntry, ResolvedExport, ResolvedImport};
use crate::codebase::symbols::output::{write_json, write_yml};
use crate::codebase::ts_symbols::ExportKind;
use std::path::PathBuf;

#[test]
fn test_write_json() {
    let mut buf = Vec::new();

    let roots = vec!["root1".to_string(), "root2".to_string()];
    let entries = vec![
        FileEntry {
            rel_path: PathBuf::from("src/foo.ts"),
            exports: vec![
                ResolvedExport {
                    name: "foo".to_string(),
                    kind: ExportKind::Function,
                    line: 10,
                    resolved: None,
                }
            ],
            imports: vec![],
        }
    ];

    write_json(&roots, &entries, &mut buf).unwrap();

    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("\"roots\":"));
    assert!(output.contains("\"root1\""));
    assert!(output.contains("\"root2\""));
    assert!(output.contains("\"files\":"));
    assert!(output.contains("\"src/foo.ts\""));
    assert!(output.contains("\"exports\":"));
    assert!(output.contains("\"foo\""));
    assert!(output.contains("\"function\""));
}

#[test]
fn test_write_yml() {
    let mut buf = Vec::new();

    let roots = vec!["root1".to_string(), "root2".to_string()];
    let entries = vec![
        FileEntry {
            rel_path: PathBuf::from("src/bar.ts"),
            exports: vec![
                ResolvedExport {
                    name: "BarClass".to_string(),
                    kind: ExportKind::Class,
                    line: 42,
                    resolved: None,
                }
            ],
            imports: vec![
                ResolvedImport {
                    source: "./baz".to_string(),
                    imported: "Baz".to_string(),
                    local: "Baz".to_string(),
                    line: 5,
                    is_type_only: false,
                    resolved: Some(PathBuf::from("src/baz.ts")),
                }
            ],
        }
    ];

    write_yml(&roots, &entries, &mut buf).unwrap();

    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("roots:"));
    assert!(output.contains("- root1"));
    assert!(output.contains("- root2"));
    assert!(output.contains("files:"));
    assert!(output.contains("path: src/bar.ts"));
    assert!(output.contains("exports:"));
    assert!(output.contains("name: BarClass"));
    assert!(output.contains("kind: class"));
    assert!(output.contains("line: 42"));
    assert!(output.contains("imports:"));
    assert!(output.contains("source: ./baz"));
    assert!(output.contains("imported: Baz"));
    assert!(output.contains("local: Baz"));
    assert!(output.contains("line: 5"));
    assert!(output.contains("resolved: src/baz.ts"));
}
