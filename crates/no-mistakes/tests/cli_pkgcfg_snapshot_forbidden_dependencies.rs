use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn cfg(name: &str) -> PathBuf {
    rules_path("forbidden-dependencies", "configs", name)
}

fn analysis_root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

#[test]
fn forbidden_dependencies_json_matches_snapshot() {
    let basic = analysis_root("forbidden-dependencies-basic");
    let globs = analysis_root("forbidden-dependencies-globs");
    assert_saved_fixture(
        "forbidden_dependencies",
        &[
            ("basic", basic.clone(), cfg("basic.yml")),
            (
                "directory_root",
                basic.clone(),
                cfg("basic-directory-root.yml"),
            ),
            (
                "relationships_all",
                basic.clone(),
                cfg("relationships-all.yml"),
            ),
            ("exclude_root", basic.clone(), cfg("exclude-root.yml")),
            ("file_exceptions", basic, cfg("file-exceptions.yml")),
            (
                "passes",
                analysis_root("forbidden-dependencies-passes"),
                cfg("passes.yml"),
            ),
            (
                "glob_module",
                analysis_root("forbidden-dependencies-glob-module"),
                cfg("glob-module.yml"),
            ),
            (
                "forbidden_file",
                analysis_root("forbidden-dependencies-file"),
                cfg("forbidden-file.yml"),
            ),
            (
                "import_type",
                analysis_root("forbidden-dependencies-relationships"),
                cfg("relationships.yml"),
            ),
            (
                "import_workspace",
                analysis_root("forbidden-dependencies-import-workspace"),
                cfg("import-workspace.yml"),
            ),
            (
                "multiple_applications",
                analysis_root("forbidden-dependencies-multi"),
                cfg("multi.yml"),
            ),
            (
                "directory_roots",
                analysis_root("forbidden-dependencies-directory"),
                cfg("directory.yml"),
            ),
            (
                "exceptions_union",
                globs.clone(),
                cfg("exceptions-union.yml"),
            ),
            ("bracket_globs", globs, cfg("bracket-globs.yml")),
            (
                "literal_directory_glob",
                analysis_root("forbidden-dependencies-globs").join("[literal"),
                cfg("literal-glob-root.yml"),
            ),
        ],
    );
}
