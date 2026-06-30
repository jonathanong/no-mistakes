use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub(crate) struct DotnetConfigProject {
    pub name: String,
    pub project: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub test: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DotnetProjectFacts {
    pub name: String,
    pub project_path: PathBuf,
    pub project_dir: PathBuf,
    pub assembly_name: String,
    pub root_namespace: String,
    pub is_test: bool,
    pub compile_files: BTreeSet<PathBuf>,
    pub project_references: BTreeSet<PathBuf>,
    pub package_references: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DotnetFileFacts {
    pub path: PathBuf,
    pub project: Option<PathBuf>,
    pub namespace: Option<String>,
    pub usings: Vec<String>,
    pub declarations: Vec<String>,
    pub references: Vec<String>,
    pub has_xunit_tests: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DotnetFactMap {
    pub projects: BTreeMap<PathBuf, DotnetProjectFacts>,
    pub files: BTreeMap<PathBuf, DotnetFileFacts>,
    pub files_by_namespace: HashMap<String, BTreeSet<PathBuf>>,
    pub declarations: HashMap<String, BTreeSet<PathBuf>>,
    pub files_by_project: HashMap<PathBuf, BTreeSet<PathBuf>>,
    pub warnings: Vec<String>,
}
