use super::text::{write_human, write_md, write_paths};
use super::{FileOutput, ImportRow, Named, Output};
use std::io::Write;

fn export_row() -> Named {
    Named {
        name: "Greeter".to_string(),
        kind: "declaration",
        line: 3,
    }
}

fn import_row() -> ImportRow {
    ImportRow {
        source: "./peer".to_string(),
        imported: "peer".to_string(),
        local: "peer".to_string(),
        line: 1,
        type_only: false,
    }
}

fn file(path: &str, exports: Vec<Named>, imports: Vec<ImportRow>) -> FileOutput {
    FileOutput {
        path: path.to_string(),
        exports,
        imports,
    }
}

fn render(
    report: &Output,
    write: impl Fn(&Output, &mut dyn Write) -> anyhow::Result<()>,
) -> String {
    let mut out = Vec::new();
    write(report, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn paths_lists_each_export_and_import() {
    let report = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", vec![export_row()], vec![import_row()])],
    };
    assert_eq!(
        render(&report, write_paths),
        "a.py:3:Greeter\na.py:1:peer\n"
    );
}

#[test]
fn human_text_covers_single_files_lists_and_empty_reports() {
    let one = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", vec![export_row()], vec![import_row()])],
    };
    assert_eq!(
        render(&one, write_human),
        "\
a.py
  export declaration  Greeter                  :3
  import peer                     :1    from ./peer
"
    );
    let split = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![
            file("a.py", vec![export_row()], Vec::new()),
            file("b.py", Vec::new(), vec![import_row()]),
        ],
    };
    assert_eq!(
        render(&split, write_human),
        "\
2 files
a.py
  export declaration  Greeter                  :3

b.py
  import peer                     :1    from ./peer
"
    );
    let empty = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    assert_eq!(render(&empty, write_human), "a.py\n  (no symbols)\n");
    let empty_many = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    assert_eq!(
        render(&empty_many, write_human),
        "2 files\n  (no symbols)\n"
    );
}

#[test]
fn markdown_covers_single_files_lists_and_empty_reports() {
    let one = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", vec![export_row()], vec![import_row()])],
    };
    assert_eq!(
        render(&one, write_md),
        "\
# `a.py`

### Exports
- `Greeter` (declaration, line 3)
### Imports
- `peer` from `./peer` (line 1)
"
    );
    let split = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![
            file("a.py", vec![export_row()], Vec::new()),
            file("b.py", Vec::new(), vec![import_row()]),
        ],
    };
    assert_eq!(
        render(&split, write_md),
        "\
# 2 files

## `a.py`
### Exports
- `Greeter` (declaration, line 3)
## `b.py`
### Imports
- `peer` from `./peer` (line 1)
"
    );
    let empty = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    assert_eq!(
        render(&empty, write_md),
        "# `a.py`\n\n_No symbols found._\n"
    );
    let empty_many = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    assert_eq!(
        render(&empty_many, write_md),
        "# 2 files\n\n_No symbols found._\n"
    );
}
