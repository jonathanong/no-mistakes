use super::{collect, file_output, resolve_format, run_symbols, write_symbols};
use crate::cli::Format;
use crate::codebase::symbols::{Include, SymbolsArgs, SymbolsMode};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends/language-symbols")
}

fn args(file: &str) -> SymbolsArgs {
    SymbolsArgs {
        files: vec![PathBuf::from(file)],
        root: Some(root()),
        tsconfig: None,
        config: None,
        mode: SymbolsMode::List,
        symbol: None,
        kinds: Vec::new(),
        include: Include::Both,
        format: Some(Format::Paths),
        json: false,
        timings: false,
    }
}

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

fn capture(args: &SymbolsArgs, format: Format) -> String {
    let mut out = Vec::new();
    write_symbols(args, format, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

fn exhaust_json(args: &SymbolsArgs) {
    let mut completed = false;
    for remaining_writes in 0..4096 {
        if write_symbols(args, Format::Json, &mut FailAfter { remaining_writes }).is_ok() {
            assert!(remaining_writes > 0);
            completed = true;
            break;
        }
    }
    assert!(completed, "json writer should succeed after enough writes");
}

#[test]
fn resolve_format_follows_json_then_explicit_then_terminal() {
    assert_eq!(
        [
            resolve_format(true, None, false),
            resolve_format(false, Some(Format::Paths), true),
            resolve_format(false, None, true),
            resolve_format(false, None, false),
        ],
        [Format::Json, Format::Paths, Format::Human, Format::Json]
    );
}

#[test]
fn writers_emit_imports_and_surface_io_errors() {
    let caller = args("python/caller.py");
    let json = capture(&caller, Format::Json);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let file = &parsed["files"][0];
    let exports = file["exports"].as_array().map(Vec::len).unwrap_or(0);
    let imports = file["imports"].as_array().map(Vec::len).unwrap_or(0);
    assert!(
        exports > 0 && imports > 0 && json.contains("caller.py"),
        "{json}"
    );
    for format in [Format::Yml, Format::Paths, Format::Md, Format::Human] {
        let text = capture(&caller, format);
        assert!(text.contains("caller.py"), "{format:?}: {text}");
        assert!(write_symbols(
            &caller,
            format,
            &mut FailAfter {
                remaining_writes: 0
            }
        )
        .is_err());
    }
    exhaust_json(&caller);
}

#[test]
fn stdout_entry_points_cover_timings_and_rejected_files() {
    let mut timed = args("python/caller.py");
    timed.timings = true;
    crate::codebase::symbols::run(timed).unwrap();

    let quiet = args("python/caller.py");
    run_symbols(quiet).unwrap();

    let json = crate::codebase::symbols::run_json(args("python/caller.py")).unwrap();
    assert!(json.contains("caller.py"), "{json}");

    let mut impact = args("python/caller.py");
    impact.mode = SymbolsMode::SignatureImpact;
    impact.symbol = Some("Greeter".to_string());
    assert!(crate::codebase::symbols::run_json(impact).is_err());
    assert!(run_symbols(args("readme.md")).is_err());
}

#[test]
fn mixed_and_non_language_files_are_rejected_before_rendering() {
    assert!(write_symbols(&args("readme.md"), Format::Json, &mut Vec::new()).is_err());
    assert!(collect(&args("readme.md")).is_err());
    let mut mixed = args("python/greeter.py");
    mixed.files.push(PathBuf::from("readme.md"));
    let error = write_symbols(&mixed, Format::Json, &mut Vec::new())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("only language files") || error.contains("only TypeScript"),
        "{error}"
    );
}

#[test]
fn file_records_follow_path_order_and_roots_stay_in_request_order() {
    let mut request = args("python/greeter.py");
    request.files = vec![
        PathBuf::from("python/greeter.py"),
        PathBuf::from("python/caller.py"),
    ];
    let report = collect(&request).unwrap();
    let paths: Vec<&str> = report.files.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(paths, ["python/caller.py", "python/greeter.py"]);
    assert_eq!(
        report.roots,
        vec![
            "python/greeter.py".to_string(),
            "python/caller.py".to_string()
        ]
    );
}

#[test]
fn missing_declaration_and_import_names_are_skipped() {
    let mut view =
        super::super::load::prepare_file(Path::new("python/greeter.py"), Some(&root()), None)
            .unwrap()
            .unwrap();
    view.declarations.push("MissingDecl".to_string());
    view.imports.push("not.in.source".to_string());
    let output = file_output(&view, true, true, true);
    assert!(
        output.exports.iter().all(|row| row.name != "MissingDecl")
            && output
                .imports
                .iter()
                .all(|row| row.local != "not.in.source")
            && !output.exports.is_empty()
    );
}
