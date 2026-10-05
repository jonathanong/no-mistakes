//! Real-PostgreSQL harness: one throwaway database per test, driven through `psql` and the CLI.
use no_mistakes::codebase::postgres::SchemaCatalog;
use no_mistakes::codebase::ts_source::{FileInventory, SourceStore};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

static CREATE_DATABASE: Mutex<()> = Mutex::new(());

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres/catalog-generation")
}

pub struct Database {
    admin: String,
    name: String,
    url: String,
}

impl Database {
    /// `None` when no cluster is configured; CI must provide one rather than skip.
    pub fn create(label: &str) -> Option<Self> {
        let Ok(admin) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") else {
            assert!(
                std::env::var_os("CI").is_none(),
                "CI must provide NO_MISTAKES_TEST_POSTGRES_URL"
            );
            return None;
        };
        let name = format!("nm_catalog_{}_{label}", std::process::id());
        let mut url = url::Url::parse(&admin).unwrap();
        url.set_path(&format!("/{name}"));
        let database = Self {
            admin,
            name,
            url: url.into(),
        };
        // CREATE DATABASE from one template cannot run concurrently; template0 is never connected to.
        let _guard = CREATE_DATABASE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        database.admin_sql(&format!(
            "DROP DATABASE IF EXISTS {} WITH (FORCE)",
            database.name
        ));
        database.admin_sql(&format!(
            "CREATE DATABASE {} TEMPLATE template0",
            database.name
        ));
        Some(database)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    fn admin_sql(&self, statement: &str) {
        run_psql(&self.admin, &["-c", statement]);
    }

    /// Run a fixture file; any SQL error fails the test.
    pub fn load(&self, file: &Path) {
        run_psql(&self.url, &["-f", file.to_str().unwrap()]);
    }

    pub fn query(&self, statement: &str) -> String {
        run_psql(&self.url, &["-c", statement])
    }

    /// Write the catalog with the CLI and return the CLI's exit output.
    pub fn generate(&self, schema: &str, coverage: Option<&str>, output: &Path) -> Output {
        self.generate_with(schema, &coverage_arguments(coverage), output)
    }

    /// Write the catalog with the CLI and extra arguments, and return the CLI's exit output.
    pub fn generate_with(&self, schema: &str, arguments: &[&str], output: &Path) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command.args([
            "postgres",
            "catalog",
            "--connection-env",
            "NM_CATALOG_TEST_URL",
        ]);
        command.args(["--schema", schema]);
        command.args(arguments);
        command.arg("--output").arg(output);
        command.env("NM_CATALOG_TEST_URL", &self.url);
        command.output().unwrap()
    }

    /// Generate and return the catalog file's bytes.
    pub fn catalog_bytes(&self, schema: &str, coverage: Option<&str>) -> Vec<u8> {
        self.catalog_bytes_with(schema, &coverage_arguments(coverage))
    }

    /// Generate with extra CLI arguments and return the catalog file's bytes.
    pub fn catalog_bytes_with(&self, schema: &str, arguments: &[&str]) -> Vec<u8> {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("catalog.json");
        let output = self.generate_with(schema, arguments, &path);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::read(path).unwrap()
    }

    pub fn catalog(&self, schema: &str, coverage: Option<&str>) -> serde_json::Value {
        serde_json::from_slice(&self.catalog_bytes(schema, coverage)).unwrap()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.admin_sql(&format!(
            "DROP DATABASE IF EXISTS {} WITH (FORCE)",
            self.name
        ));
    }
}

fn coverage_arguments(coverage: Option<&str>) -> Vec<&str> {
    coverage.map_or_else(Vec::new, |coverage| vec!["--coverage", coverage])
}

fn run_psql(url: &str, arguments: &[&str]) -> String {
    let output = Command::new("psql")
        .args(["-X", "--no-password", "-q", "-At", "-v", "ON_ERROR_STOP=1"])
        .args(arguments)
        .arg(url)
        .output()
        .expect("psql must be installed");
    assert!(
        output.status.success(),
        "psql {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Load a generated catalog through the normal reader.
pub fn load_catalog(directory: &Path, file: &str) -> SchemaCatalog {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[directory.join(file)])));
    SchemaCatalog::load(directory, file, &sources).unwrap()
}

/// Copy a committed fixture tree so a test can add the catalog it generated beside the sources.
pub fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// `check --format json` findings for a project: (rule, file, message) triples.
pub fn check(root: &Path, config: &Path) -> (Output, Vec<(String, String, String)>) {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let findings = report["rules"]
        .as_array()
        .map(|rules| {
            rules
                .iter()
                .map(|finding| {
                    (
                        finding["rule"].as_str().unwrap().to_string(),
                        finding["file"].as_str().unwrap().to_string(),
                        finding["message"].as_str().unwrap().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    (output, findings)
}
