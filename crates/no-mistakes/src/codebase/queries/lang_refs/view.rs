use super::lines;
use super::load::{rel, Corpus};
use crate::codebase::dependencies::graph::EdgeKind;
use crate::codebase::lang_frontends::LangFactMap;
use crate::codebase::ts_source::SourceStore;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct View {
    pub root: PathBuf,
    pub rel_file: String,
    pub declarations: Vec<String>,
    pub imports: Vec<String>,
    pub sources: Arc<SourceStore>,
    abs_file: PathBuf,
    facts: Facts,
}

enum Facts {
    Lang(LangFactMap, EdgeKind, EdgeKind),
    Swift(crate::codebase::swift::SwiftFactMap),
    Dotnet(crate::codebase::dotnet::DotnetFactMap),
}

impl View {
    pub(super) fn unread_files(&self) -> Vec<PathBuf> {
        match &self.facts {
            Facts::Lang(facts, _, _) => facts.unread_files.iter().cloned().collect(),
            Facts::Swift(facts) => facts.unread_files.iter().cloned().collect(),
            Facts::Dotnet(facts) => facts.unread_files.iter().cloned().collect(),
        }
    }

    pub(super) fn reference_files(&self, name: &str) -> BTreeSet<PathBuf> {
        let graph = crate::codebase::dependencies::graph::proven_lang_reference_files;
        match &self.facts {
            Facts::Lang(facts, import_kind, ref_kind) => {
                graph(facts, &self.abs_file, name, *import_kind, *ref_kind)
            }
            Facts::Swift(facts) => {
                crate::codebase::dependencies::graph::proven_swift_reference_files(
                    facts,
                    &self.abs_file,
                    name,
                )
            }
            Facts::Dotnet(facts) => {
                crate::codebase::dependencies::graph::proven_dotnet_reference_files(
                    facts,
                    &self.abs_file,
                    name,
                )
            }
        }
    }

    pub(super) fn declaration_line(&self, name: &str) -> Option<u32> {
        lines::first_name_line(&self.sources, &self.abs_file, name)
    }

    pub(super) fn import_line(&self, specifier: &str) -> Option<u32> {
        lines::first_import_line(&self.sources, &self.abs_file, specifier)
    }

    pub(super) fn swift(
        corpus: &Corpus,
        abs_file: &Path,
        declarations: Vec<String>,
        imports: Vec<String>,
        facts: crate::codebase::swift::SwiftFactMap,
    ) -> Self {
        Self::assemble(corpus, abs_file, declarations, imports, Facts::Swift(facts))
    }

    pub(super) fn dotnet(
        corpus: &Corpus,
        abs_file: &Path,
        declarations: Vec<String>,
        imports: Vec<String>,
        facts: crate::codebase::dotnet::DotnetFactMap,
    ) -> Self {
        Self::assemble(
            corpus,
            abs_file,
            declarations,
            imports,
            Facts::Dotnet(facts),
        )
    }

    pub(super) fn lang(
        corpus: &Corpus,
        abs_file: &Path,
        declarations: Vec<String>,
        imports: Vec<String>,
        facts: LangFactMap,
        import_kind: EdgeKind,
        ref_kind: EdgeKind,
    ) -> Self {
        Self::assemble(
            corpus,
            abs_file,
            declarations,
            imports,
            Facts::Lang(facts, import_kind, ref_kind),
        )
    }

    fn assemble(
        corpus: &Corpus,
        abs_file: &Path,
        declarations: Vec<String>,
        imports: Vec<String>,
        facts: Facts,
    ) -> Self {
        Self {
            root: corpus.root.clone(),
            rel_file: rel(abs_file, &corpus.root),
            declarations,
            imports,
            sources: Arc::clone(&corpus.sources),
            abs_file: abs_file.to_path_buf(),
            facts,
        }
    }
}
