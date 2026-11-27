use super::super::shared::{resolve_target, Target};
use super::kind::Frontend;
use super::native;
use super::view::View;

pub(super) use super::kind::frontend;
use crate::codebase::dependencies::graph::EdgeKind;
use crate::codebase::lang_frontends::{lang_config_from_v2, CollectedLangFacts};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Corpus {
    pub root: PathBuf,
    pub files: Arc<Vec<PathBuf>>,
    pub sources: Arc<SourceStore>,
    pub config: NoMistakesConfig,
    collected: Arc<CollectedLangFacts>,
}

pub(super) fn prepare_file(
    file: &Path,
    root: Option<&Path>,
    config: Option<&Path>,
) -> Result<Option<View>> {
    let Some(kind) = frontend(file) else {
        return Ok(None);
    };
    let target = resolve_target(file, root, None)?;
    let corpus = open_corpus(&target, config)?;
    if !native::configured(&corpus.config, &kind) {
        bail!(
            "{} is not in a configured {} package list",
            rel(&target.abs_file, &corpus.root),
            kind.label()
        );
    }
    let view = match kind {
        Frontend::Swift => native::swift_view(&corpus, &target.abs_file)?,
        Frontend::Dotnet => native::dotnet_view(&corpus, &target.abs_file)?,
        other => lang_view(&corpus, &target.abs_file, other)?,
    };
    Ok(Some(view))
}

fn open_corpus(target: &Target, config: Option<&Path>) -> Result<Corpus> {
    let config = target.config(config).context("loading .no-mistakes.yml")?;
    let mut files = crate::codebase::ts_source::discover_files_from_visible(
        &target.root,
        &[],
        target.visible_paths.paths_for(&target.root).as_ref(),
    );
    crate::codebase::ts_source::sort_os_str_paths(&mut files);
    files.dedup();
    let files = Arc::new(files);
    let collected = target.session.lang_frontend_facts(
        &target.root,
        &files,
        &lang_config_from_v2(config.as_ref()),
        &target.sources,
    );
    Ok(Corpus {
        root: target.root.clone(),
        files,
        sources: Arc::clone(&target.sources),
        config: (*config).clone(),
        collected,
    })
}

fn lang_view(corpus: &Corpus, abs_file: &Path, kind: Frontend) -> Result<View> {
    let (facts, import_kind, ref_kind) = match &kind {
        Frontend::Python => (
            &corpus.collected.python,
            EdgeKind::PythonImport,
            EdgeKind::PythonReference,
        ),
        Frontend::Go => (
            &corpus.collected.go,
            EdgeKind::GoImport,
            EdgeKind::GoReference,
        ),
        Frontend::Rust => (&corpus.collected.rust, EdgeKind::RustUse, EdgeKind::RustUse),
        Frontend::Ruby => (
            &corpus.collected.ruby,
            EdgeKind::RubyRequire,
            EdgeKind::RubyReference,
        ),
        Frontend::Php => (&corpus.collected.php, EdgeKind::PhpUse, EdgeKind::PhpUse),
        Frontend::Java => (
            &corpus.collected.java,
            EdgeKind::JavaImport,
            EdgeKind::JavaReference,
        ),
        Frontend::Kotlin => (
            &corpus.collected.kotlin,
            EdgeKind::KotlinImport,
            EdgeKind::KotlinReference,
        ),
        Frontend::Elixir => (
            &corpus.collected.elixir,
            EdgeKind::ElixirImport,
            EdgeKind::ElixirReference,
        ),
        Frontend::Dart => (
            &corpus.collected.dart,
            EdgeKind::DartImport,
            EdgeKind::DartReference,
        ),
        Frontend::Swift | Frontend::Dotnet => {
            bail!("native language files use their own fact maps")
        }
    };
    let Some(file) = facts.files.get(abs_file) else {
        bail!(
            "{} is outside the configured {} packages",
            rel(abs_file, &corpus.root),
            kind.label()
        );
    };
    Ok(View::lang(
        corpus,
        abs_file,
        file.declarations.clone(),
        file.imports.clone(),
        facts.clone(),
        import_kind,
        ref_kind,
    ))
}

pub(super) fn rel(path: &Path, root: &Path) -> String {
    crate::codebase::queries::shared::rel_str(path, root)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod native_bail;
