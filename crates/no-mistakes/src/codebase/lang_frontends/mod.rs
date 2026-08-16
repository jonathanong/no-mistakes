mod facts;
mod go;
mod kafka;
mod php;
mod python;
mod ruby;
mod rustlang;
mod strip;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub(crate) use facts::LangFactMap;
pub(crate) use go::collect_go_facts;
pub(crate) use kafka::{scan_file as scan_kafka_file, topic_identity};
pub(crate) use php::collect_php_facts;
pub(crate) use python::collect_python_facts;
pub(crate) use ruby::collect_ruby_facts;
pub(crate) use rustlang::collect_rust_facts;

#[derive(Debug, Clone, Default)]
pub(crate) struct LangFrontendConfig {
    pub python_packages: Vec<String>,
    pub go_modules: Vec<String>,
    pub rust_packages: Vec<String>,
    pub rails_apps: Vec<String>,
    pub php_apps: Vec<String>,
}

pub(crate) struct CollectedLangFacts {
    pub python: LangFactMap,
    pub go: LangFactMap,
    pub rust: LangFactMap,
    pub ruby: LangFactMap,
    pub php: LangFactMap,
}

pub(crate) fn collect_all_lang_facts(
    root: &Path,
    all_files: &[PathBuf],
    config: &LangFrontendConfig,
) -> CollectedLangFacts {
    CollectedLangFacts {
        python: collect_python_facts(root, all_files, &config.python_packages),
        go: collect_go_facts(root, all_files, &config.go_modules),
        rust: collect_rust_facts(root, all_files, &config.rust_packages),
        ruby: collect_ruby_facts(root, all_files, &config.rails_apps),
        php: collect_php_facts(root, all_files, &config.php_apps),
    }
}
