use super::super::kind::Frontend;
use super::{lang_view, Corpus};
use crate::codebase::lang_frontends::CollectedLangFacts;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn swift_and_dotnet_do_not_use_the_shared_language_map() {
    let corpus = Corpus {
        root: PathBuf::from("root"),
        files: Arc::new(Vec::new()),
        sources: Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[])))),
        config: NoMistakesConfig::default(),
        collected: Arc::new(CollectedLangFacts::default()),
    };
    let file = PathBuf::from("Native.swift");
    let swift = match lang_view(&corpus, &file, Frontend::Swift) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("swift must not use the shared language map"),
    };
    let dotnet = match lang_view(&corpus, &file, Frontend::Dotnet) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("dotnet must not use the shared language map"),
    };
    assert!(
        swift.contains("native language files") && dotnet.contains("native language files"),
        "{swift} / {dotnet}"
    );
}
