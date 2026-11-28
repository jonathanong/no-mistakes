use super::*;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

struct StubSources {
    reads: AtomicUsize,
    source_reads: AtomicUsize,
    json_reads: AtomicUsize,
}

impl SourceAccess for StubSources {
    fn read_path(&self, _path: &Path) -> SourceReadOutcome {
        self.reads.fetch_add(1, Ordering::Relaxed);
        Ok(Arc::from("plain"))
    }

    fn read_source_path(&self, _path: &Path) -> SourceReadOutcome {
        self.source_reads.fetch_add(1, Ordering::Relaxed);
        Ok(Arc::from("export const x = 1;"))
    }

    fn parse_json_path(&self, _path: &Path) -> JsonParseOutcome {
        self.json_reads.fetch_add(1, Ordering::Relaxed);
        Ok(Arc::new(serde_json::json!({})))
    }
}

#[test]
fn json_load_error_formats_and_exposes_underlying_source() {
    let io_err = JsonLoadError::Io(Arc::new(io::Error::new(io::ErrorKind::NotFound, "missing")));
    assert!(io_err.to_string().contains("missing"));
    assert!(io_err.source().is_some());

    let syntax_err = JsonLoadError::Syntax(Arc::new(
        serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
    ));
    assert!(!syntax_err.to_string().is_empty());
    assert!(syntax_err.source().is_some());
}

#[test]
fn read_prepared_or_open_delegates_through_arc_and_disk_fallback() {
    let stub = Arc::new(StubSources {
        reads: AtomicUsize::new(0),
        source_reads: AtomicUsize::new(0),
        json_reads: AtomicUsize::new(0),
    });
    assert_eq!(
        read_prepared_or_open(Some(&stub), Path::new("app.ts"))
            .unwrap()
            .as_ref(),
        "export const x = 1;"
    );
    assert_eq!(
        read_prepared_or_open(Some(&stub), Path::new("config.json"))
            .unwrap()
            .as_ref(),
        "plain"
    );
    assert!(stub.parse_json_path(Path::new("config.json")).is_ok());
    assert_eq!(stub.source_reads.load(Ordering::Relaxed), 1);
    assert_eq!(stub.reads.load(Ordering::Relaxed), 1);
    assert_eq!(stub.json_reads.load(Ordering::Relaxed), 1);

    let json_fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution/tsconfig.json");
    let text = read_prepared_or_open(None, &json_fixture).unwrap();
    assert!(text.contains('{'));

    let ts_fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution/apps/web/src/entry.ts");
    let ts_text = read_prepared_or_open(None, &ts_fixture).unwrap();
    assert!(!ts_text.is_empty());
}
