use crate::codebase::postgres::query_annotation::evaluate::File;
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn empty_files<'a>(
    paths: &[PathBuf],
    facts: &'a QueryAnnotationFileFacts,
) -> (Arc<ModuleIds>, FxHashMap<PathBuf, File<'a>>) {
    let ids = Arc::new(ModuleIds::new(paths.iter().cloned()));
    let module =
        Arc::new(Module::new(Arc::new(TsFileFacts::default())).with_module_ids(Arc::clone(&ids)));
    let mut files = FxHashMap::default();
    for path in paths {
        files.insert(
            path.clone(),
            File {
                facts,
                module: Arc::clone(&module),
                executors: FxHashSet::default(),
            },
        );
    }
    (ids, files)
}
