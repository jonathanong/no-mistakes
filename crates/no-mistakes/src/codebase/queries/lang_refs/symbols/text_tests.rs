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

struct FailAfter {
    remaining_writes: usize,
}

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.remaining_writes == 0 {
            return Err(std::io::Error::other("synthetic write failure"));
        }
        self.remaining_writes -= 1;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn exhaust(mut write: impl FnMut(&mut FailAfter) -> anyhow::Result<()>) {
    let mut completed = false;
    for remaining_writes in 0..4096 {
        if write(&mut FailAfter { remaining_writes }).is_ok() {
            assert!(remaining_writes > 0);
            completed = true;
            break;
        }
    }
    assert!(completed, "writer should succeed after enough writes");
}

#[test]
fn text_writers_surface_io_errors() {
    let populated = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", vec![export_row()], vec![import_row()])],
    };
    let split = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![
            file("a.py", vec![export_row()], Vec::new()),
            file("b.py", Vec::new(), vec![import_row()]),
        ],
    };
    let empty = Output {
        roots: vec!["a.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    let empty_many = Output {
        roots: vec!["a.py".to_string(), "b.py".to_string()],
        files: vec![file("a.py", Vec::new(), Vec::new())],
    };
    exhaust(|writer| write_paths(&populated, writer));
    exhaust(|writer| write_human(&populated, writer));
    exhaust(|writer| write_human(&split, writer));
    exhaust(|writer| write_human(&empty, writer));
    exhaust(|writer| write_human(&empty_many, writer));
    exhaust(|writer| write_md(&populated, writer));
    exhaust(|writer| write_md(&split, writer));
    exhaust(|writer| write_md(&empty, writer));
    exhaust(|writer| write_md(&empty_many, writer));
}
