use super::*;
use crate::codebase::rules::postgres_column_requires_trigger::RULE_ID as COLUMN_RULE;
use crate::codebase::rules::postgres_conflict_ordering::RULE_ID as CONFLICT_RULE;
use crate::codebase::rules::postgres_duplicate_function_body::RULE_ID as DUPLICATE_RULE;
use crate::codebase::rules::postgres_required_comments::RULE_ID as COMMENTS_RULE;
use crate::codebase::rules::postgres_table_shape::RULE_ID as TABLE_RULE;
use crate::codebase::ts_source::FileInventory;

#[test]
fn missing_facts_use_the_file_checkers() {
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let config = NoMistakesConfig::default();
    let files: &[PathBuf] = &[];
    for rule_id in [
        DUPLICATE_RULE,
        COMMENTS_RULE,
        COLUMN_RULE,
        CONFLICT_RULE,
        TABLE_RULE,
    ] {
        assert!(run(rule_id, Path::new("."), &config, files, &sources, None).is_some());
    }
}
