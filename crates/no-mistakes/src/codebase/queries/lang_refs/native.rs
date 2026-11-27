use super::kind::Frontend;
use super::load::{rel, Corpus};
use super::view::View;
use crate::codebase::dotnet::collect_dotnet_facts_with_sources;
use crate::codebase::swift::collect_swift_facts_with_sources;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub(super) fn configured(config: &NoMistakesConfig, kind: &Frontend) -> bool {
    let tests = &config.tests;
    match kind {
        Frontend::Python => !tests.python.packages.is_empty(),
        Frontend::Go => !tests.go.modules.is_empty(),
        Frontend::Rust => !tests.rust.packages.is_empty(),
        Frontend::Ruby => !tests.rails.apps.is_empty(),
        Frontend::Php => !tests.php.apps.is_empty(),
        Frontend::Java => !tests.java.packages.is_empty(),
        Frontend::Kotlin => !tests.kotlin.packages.is_empty(),
        Frontend::Elixir => !tests.elixir.apps.is_empty(),
        Frontend::Dart => !tests.dart.packages.is_empty(),
        Frontend::Swift => !tests.swift.packages.is_empty(),
        Frontend::Dotnet => !tests.dotnet.projects.is_empty() || !tests.dotnet.solutions.is_empty(),
    }
}

pub(super) fn swift_view(corpus: &Corpus, abs_file: &Path) -> Result<View> {
    let facts = collect_swift_facts_with_sources(
        &corpus.root,
        corpus.files.as_slice(),
        &corpus.config.tests.swift.packages,
        Some(&corpus.sources),
    );
    let Some(file) = facts.files.get(abs_file) else {
        bail!(
            "{} is outside the configured swift packages",
            rel(abs_file, &corpus.root)
        );
    };
    Ok(View::swift(
        corpus,
        abs_file,
        file.declarations.clone(),
        file.imports.clone(),
        facts,
    ))
}

pub(super) fn dotnet_view(corpus: &Corpus, abs_file: &Path) -> Result<View> {
    let projects =
        crate::codebase::dotnet::configured_projects(&corpus.root, &corpus.config.tests.dotnet);
    let facts = collect_dotnet_facts_with_sources(
        &corpus.root,
        corpus.files.as_slice(),
        &projects,
        Some(&corpus.sources),
    );
    let Some(file) = facts.files.get(abs_file) else {
        bail!(
            "{} is outside the configured dotnet projects",
            rel(abs_file, &corpus.root)
        );
    };
    Ok(View::dotnet(
        corpus,
        abs_file,
        file.declarations.clone(),
        file.usings.clone(),
        facts,
    ))
}
