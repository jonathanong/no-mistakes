mod cli;
mod dart;
mod elixir;
mod facts;
mod go;
mod java;
mod kafka;
mod kotlin;
mod php;
mod python;
mod ruby;
mod rustlang;
mod strip;
mod partition;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_collect;
#[cfg(test)]
mod tests_extra;
#[cfg(test)]
mod tests_more;
#[cfg(test)]
mod tests_p2;
#[cfg(test)]
mod tests_p3;

use std::path::{Path, PathBuf};

pub(crate) use cli::{
    each_lang_map, lang_config_from_v2, lang_config_is_empty, matching_cluster,
    queue_globs_from_v2, QueueGlobMatchers,
};
pub(crate) use dart::{collect_dart_facts, extract_http_paths};
pub(crate) use elixir::collect_elixir_facts;
pub(crate) use facts::{configured_roots, LangFactMap, LangFileFacts};
pub(crate) use go::collect_go_facts;
pub(crate) use java::collect_java_facts;
pub(crate) use kafka::{scan_file as scan_kafka_file, topic_identity};
pub(crate) use kotlin::collect_kotlin_facts;
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
    pub php_framework: Option<String>,
    pub java_packages: Vec<String>,
    pub kotlin_packages: Vec<String>,
    pub elixir_apps: Vec<String>,
    pub dart_packages: Vec<String>,
}

#[derive(Default)]
pub(crate) struct CollectedLangFacts {
    pub python: LangFactMap,
    pub go: LangFactMap,
    pub rust: LangFactMap,
    pub ruby: LangFactMap,
    pub php: LangFactMap,
    pub java: LangFactMap,
    pub kotlin: LangFactMap,
    pub elixir: LangFactMap,
    pub dart: LangFactMap,
}

pub(crate) fn collect_all_lang_facts(
    root: &Path,
    all_files: &[PathBuf],
    config: &LangFrontendConfig,
    sources: &crate::codebase::ts_source::SourceStore,
) -> CollectedLangFacts {
    let parts = partition::partition_files_by_extension(all_files);
    // Each collect_*_facts already file-parallelizes. Overlapping language
    // extractors with nested rayon::join raised language_frontends::extract
    // peak memory 260.8 KB → 688.5 KB, past the extra-join ≤10% memory gate.
    CollectedLangFacts {
        python: collect_python_facts(root, &parts.py, &config.python_packages, sources),
        go: collect_go_facts(root, &parts.go, &config.go_modules, sources),
        rust: collect_rust_facts(root, &parts.rs, &config.rust_packages, sources),
        ruby: collect_ruby_facts(root, &parts.rb, &config.rails_apps, sources),
        php: collect_php_facts(
            root,
            &parts.php_universe(),
            &config.php_apps,
            config.php_framework.as_deref(),
            sources,
        ),
        java: collect_java_facts(root, &parts.java, &config.java_packages, sources),
        kotlin: collect_kotlin_facts(root, &parts.kt, &config.kotlin_packages, sources),
        elixir: collect_elixir_facts(root, &parts.elixir, &config.elixir_apps, sources),
        dart: collect_dart_facts(root, &parts.dart, &config.dart_packages, sources),
    }
}
