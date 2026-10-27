use super::*;

// Output cases moved to cli_rra_snapshot_banned_renamed_files. Keep regex
// compile errors and paths the walker does not treat as files.

fn compiled(opts: &Options) -> Vec<CompiledBasename<'_>> {
    compile_basenames(opts).unwrap()
}

fn opts_regex(pattern: &str) -> Options {
    Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "jest.config".to_string(),
            message: "remove jest config".to_string(),
            pattern: Some(pattern.to_string()),
            ..Default::default()
        }],
        // extensions intentionally narrow — regex entries ignore the extension filter.
        extensions: vec![".ts".to_string()],
    }
}

#[test]
fn invalid_regex_pattern_errors() {
    let opts = opts_regex(r"(unclosed");
    let err = match compile_basenames(&opts) {
        Ok(_) => panic!("expected invalid regex to error"),
        Err(err) => err,
    };
    let msg = format!("{err:#}");
    assert!(msg.contains("invalid `pattern` regex"), "got: {msg}");
    assert!(
        msg.contains("jest.config"),
        "error names the basename: {msg}"
    );
}

#[test]
fn file_with_no_name_component_returns_empty() {
    // A path with no file_name (e.g. the root itself) should return no findings.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    // Pass the root dir itself — has no file_name in the sense of "middleware"
    let findings = check_file(root, root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "root path (no matching stem) should produce no findings"
    );
}

#[test]
fn path_with_no_file_name_returns_empty() {
    // On Unix, Path::new("/").file_name() returns None, exercising line 81.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        scope: None,
        banned_basenames: vec![BannedBasename {
            name: "middleware".to_string(),
            message: "rename it".to_string(),
            ..Default::default()
        }],
        extensions: vec![".ts".to_string()],
    };
    // Use the filesystem root — file_name() is None for paths ending in "/"
    let findings = check_file(std::path::Path::new("/"), root, &opts, &compiled(&opts));
    assert!(
        findings.is_empty(),
        "path with no file_name should return no findings"
    );
}
