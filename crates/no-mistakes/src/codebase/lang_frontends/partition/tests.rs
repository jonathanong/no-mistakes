use super::partition_files_by_extension;
use std::path::PathBuf;

#[test]
fn partition_files_by_extension_buckets_go_mod_and_php_universe() {
    let files = [
        PathBuf::from("app.py"),
        PathBuf::from("go.mod"),
        PathBuf::from("pkg/main.go"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("app.rb"),
        PathBuf::from("index.php"),
        PathBuf::from("composer.json"),
        PathBuf::from("routes.yaml"),
        PathBuf::from("services.yml"),
        PathBuf::from("Main.java"),
        PathBuf::from("App.kt"),
        PathBuf::from("lib/app.ex"),
        PathBuf::from("test/app_test.exs"),
        PathBuf::from("notes.md"),
    ];
    let parts = partition_files_by_extension(&files);
    assert_eq!(parts.py, vec![PathBuf::from("app.py")]);
    assert_eq!(
        parts.go,
        vec![PathBuf::from("go.mod"), PathBuf::from("pkg/main.go")]
    );
    assert_eq!(parts.rs, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(parts.rb, vec![PathBuf::from("app.rb")]);
    assert_eq!(parts.php, vec![PathBuf::from("index.php")]);
    assert_eq!(parts.json, vec![PathBuf::from("composer.json")]);
    assert_eq!(parts.yaml, vec![PathBuf::from("routes.yaml")]);
    assert_eq!(parts.yml, vec![PathBuf::from("services.yml")]);
    assert_eq!(parts.java, vec![PathBuf::from("Main.java")]);
    assert_eq!(parts.kt, vec![PathBuf::from("App.kt")]);
    assert_eq!(
        parts.elixir,
        vec![
            PathBuf::from("lib/app.ex"),
            PathBuf::from("test/app_test.exs")
        ]
    );
    let universe = parts.php_universe();
    assert_eq!(
        universe,
        vec![
            PathBuf::from("index.php"),
            PathBuf::from("composer.json"),
            PathBuf::from("routes.yaml"),
            PathBuf::from("services.yml"),
        ]
    );
}
