use super::kind::Frontend;
use super::native;
use super::view::View;

pub(super) use super::kind::frontend;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::{EdgeKind, GraphFiles};
use crate::codebase::lang_frontends::{lang_config_from_v2, CollectedLangFacts};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::{load_v2_config, NoMistakesConfig};
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
    let corpus = open_corpus(root, config)?;
    if !native::configured(&corpus.config, &kind) {
        bail!(
            "{} is not in a configured {} package list",
            rel(&abs_under(&corpus.root, file), &corpus.root),
            kind.label()
        );
    }
    let abs_file = abs_under(&corpus.root, file);
    let view = match kind {
        Frontend::Swift => native::swift_view(&corpus, &abs_file)?,
        Frontend::Dotnet => native::dotnet_view(&corpus, &abs_file)?,
        other => lang_view(&corpus, &abs_file, other)?,
    };
    Ok(Some(view))
}

fn open_corpus(root: Option<&Path>, config: Option<&Path>) -> Result<Corpus> {
    let root = resolve_root(root)?;
    let config = load_v2_config(&root, config).context("loading .no-mistakes.yml")?;
    let files = Arc::new(GraphFiles::discover(&root).all().to_vec());
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        files.as_slice(),
    ))));
    let session = AnalysisSession::new(crate::diagnostics::current());
    let collected =
        session.lang_frontend_facts(&root, &files, &lang_config_from_v2(&config), &sources);
    Ok(Corpus {
        root,
        files,
        sources,
        config,
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

fn resolve_root(root: Option<&Path>) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    Ok(normalize_path(&match root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd,
    }))
}

fn abs_under(root: &Path, file: &Path) -> PathBuf {
    if file.is_absolute() {
        normalize_path(file)
    } else {
        normalize_path(&root.join(file))
    }
}
