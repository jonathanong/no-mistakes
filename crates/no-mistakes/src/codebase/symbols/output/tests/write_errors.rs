use crate::codebase::symbols::output::{write_human, write_json, write_md, write_paths, write_yml};
use crate::codebase::symbols::{FileEntry, ResolvedExport, ResolvedImport};
use crate::codebase::ts_symbols::ExportKind;
use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;

struct FailAfter {
    remaining_writes: usize,
}

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining_writes == 0 {
            return Err(io::Error::other("synthetic write failure"));
        }
        self.remaining_writes -= 1;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn exhaust_write(mut write: impl FnMut(&mut FailAfter) -> Result<()>) {
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

fn populated_entry() -> FileEntry {
    FileEntry {
        rel_path: PathBuf::from("src/foo.ts"),
        exports: vec![
            ResolvedExport {
                name: "foo".to_string(),
                kind: ExportKind::Function,
                line: 1,
                resolved: None,
            },
            ResolvedExport {
                name: "reexported".to_string(),
                kind: ExportKind::ReExport {
                    source: "./bar".to_string(),
                    imported: "bar".to_string(),
                },
                line: 2,
                resolved: Some(PathBuf::from("src/bar.ts")),
            },
        ],
        imports: vec![
            ResolvedImport {
                source: "./bar".to_string(),
                imported: "Bar".to_string(),
                local: "Bar".to_string(),
                line: 3,
                is_type_only: false,
                resolved: Some(PathBuf::from("src/bar.ts")),
            },
            ResolvedImport {
                source: "lodash".to_string(),
                imported: "map".to_string(),
                local: "mapFn".to_string(),
                line: 4,
                is_type_only: true,
                resolved: None,
            },
        ],
    }
}

#[test]
fn symbol_output_writers_surface_io_errors() {
    let empty = FileEntry {
        rel_path: PathBuf::from("src/empty.ts"),
        exports: vec![],
        imports: vec![],
    };
    let entries = [populated_entry(), empty.clone()];
    let roots = ["src/foo.ts".to_string(), "src/empty.ts".to_string()];
    exhaust_write(|writer| write_json(&roots, &entries, writer));
    exhaust_write(|writer| write_yml(&roots, &entries, writer));
    exhaust_write(|writer| write_md(&roots, &entries, writer));
    exhaust_write(|writer| write_md(&["src/foo.ts".to_string()], &[], writer));
    exhaust_write(|writer| write_human(&roots, &entries, writer));
    exhaust_write(|writer| {
        write_human(
            &["src/foo.ts".to_string()],
            std::slice::from_ref(&empty),
            writer,
        )
    });
    exhaust_write(|writer| write_paths(&entries, writer));
}
