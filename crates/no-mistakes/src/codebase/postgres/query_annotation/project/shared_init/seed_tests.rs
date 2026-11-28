use super::super::super::evaluate::module_cache::shared::SharedEntry;
use super::super::super::evaluate::File;
use super::super::super::module_ids::ModuleIds;
use super::super::super::modules::Module;
use super::super::super::QueryAnnotationFileFacts;
use crate::codebase::check_facts::sql_source_prerequisites::FailedSqlSources;
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::sync::Arc;

fn make_file<'a>(
    facts: &'a QueryAnnotationFileFacts,
    specifiers: Vec<&str>,
    ids: &Arc<ModuleIds>,
) -> File<'a> {
    let ts = TsFileFacts {
        imports: specifiers
            .into_iter()
            .map(|specifier| ExtractedImport {
                specifier: specifier.into(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: None,
                function_scope_id: None,
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
                computed: false,
            })
            .collect(),
        ..Default::default()
    };
    File {
        facts,
        module: Arc::new(Module::new(Arc::new(ts)).with_module_ids(Arc::clone(ids))),
        executors: FxHashSet::default(),
    }
}

#[test]
fn second_seed_picks_root_reaching_uncaptured_module_and_skips_unrelated_roots() {
    let r_a = PathBuf::from("root-a.mts");
    let r_b = PathBuf::from("root-b.mts");
    let r_c = PathBuf::from("root-c.mts");
    let helper = PathBuf::from("helper.mts");
    let ids = Arc::new(ModuleIds::new([
        r_a.clone(),
        r_b.clone(),
        r_c.clone(),
        helper.clone(),
    ]));
    let facts = QueryAnnotationFileFacts {
        precise_records: true,
        ..Default::default()
    };
    let mut files = FxHashMap::default();
    files.insert(r_a.clone(), make_file(&facts, vec!["./helper.mts"], &ids));
    files.insert(r_b.clone(), make_file(&facts, vec![], &ids));
    files.insert(r_c.clone(), make_file(&facts, vec!["./helper.mts"], &ids));
    files.insert(helper.clone(), make_file(&facts, vec![], &ids));
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helper.mts").then(|| helper.clone())
    };
    let mut roots = vec![r_a, r_b.clone(), r_c.clone()];
    let status = super::build(
        &files,
        &roots,
        &resolve,
        false,
        &FailedSqlSources::default(),
        None,
    );

    // First seed picks root-a (index 0, covers helper).
    assert_eq!(status.next_seed_root(&roots), Some(0));
    roots.remove(0);

    // Remaining roots are [root-b (unrelated), root-c (reaches helper)].
    // Second seed must skip unrelated root-b and select root-c (index 1).
    assert_eq!(status.next_seed_root(&roots), Some(1));
    roots.remove(1);

    // Only unrelated root-b remains; no second/further seed reaches uncaptured helper.
    assert_eq!(status.next_seed_root(&roots), None);

    // Control: root-c still covers the uncaptured helper before it is recorded.
    assert_eq!(status.next_seed_root(std::slice::from_ref(&r_c)), Some(0));
    // Once all eligible modules are recorded, even a covering root returns None.
    status
        .shared()
        .unwrap()
        .entries
        .insert(ids.id(&helper), SharedEntry::Impure);
    assert_eq!(status.next_seed_root(std::slice::from_ref(&r_c)), None);
}
