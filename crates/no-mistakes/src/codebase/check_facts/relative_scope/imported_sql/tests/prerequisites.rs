use super::*;
use crate::codebase::check_facts::sql_source_prerequisites::{SqlProjection, SqlSourceKey};

#[test]
fn failed_reexport_source_survives_cached_lookup_and_completion() {
    let mut files = prepared("imported-barrel");
    let barrel = files
        .keys()
        .find(|path| path.ends_with("src/helpers/index.mts"))
        .cloned()
        .unwrap();
    Arc::make_mut(&mut files.get_mut(&barrel).unwrap().ts).fatal_parse_error = true;
    let calls = Mutex::new(BTreeMap::new());
    let mut locator = Locator::new(&files, resolver(&files, &calls));
    let first = locator.locate(&barrel, "ownerVisibleSql");
    assert!(locator.failures.contains(&barrel));
    locator.failures.clear();
    assert_eq!(locator.locate(&barrel, "ownerVisibleSql"), first);
    assert!(locator.failures.contains(&barrel));
    drop(locator);
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    let caller = (&files)
        .into_iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .unwrap()
        .1;
    let key = SqlSourceKey::profile(SqlProjection::Embedded, &caller.embedded_sql[0].0);
    assert!(caller
        .sql_source_prerequisites
        .failures(&key)
        .any(|path| path == &barrel));
}
