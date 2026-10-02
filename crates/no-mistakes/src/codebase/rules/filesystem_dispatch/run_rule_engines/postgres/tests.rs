use super::*;
use crate::codebase::rules::postgres_array_columns::RULE_ID as ARRAY_COLUMNS_RULE;
use crate::codebase::rules::postgres_column_naming::RULE_ID as COLUMN_NAMING_RULE;
use crate::codebase::rules::postgres_column_requires_trigger::RULE_ID as COLUMN_RULE;
use crate::codebase::rules::postgres_conflict_ordering::RULE_ID as CONFLICT_RULE;
use crate::codebase::rules::postgres_duplicate_function_body::RULE_ID as DUPLICATE_RULE;
use crate::codebase::rules::postgres_explicit_columns::RULE_ID as EXPLICIT_COLUMNS_RULE;
use crate::codebase::rules::postgres_finite_text_columns::RULE_ID as FINITE_TEXT_RULE;
use crate::codebase::rules::postgres_identifier_length::RULE_ID as IDENTIFIER_LENGTH_RULE;
use crate::codebase::rules::postgres_no_offset::RULE_ID as NO_OFFSET_RULE;
use crate::codebase::rules::postgres_object_naming::RULE_ID as NAMING_RULE;
use crate::codebase::rules::postgres_required_comments::RULE_ID as COMMENTS_RULE;
use crate::codebase::rules::postgres_required_predicates::RULE_ID as REQUIRED_PREDICATES_RULE;
use crate::codebase::rules::postgres_status_with_lifecycle_timestamps::RULE_ID as STATUS_RULE;
use crate::codebase::rules::postgres_table_shape::RULE_ID as TABLE_RULE;
use crate::codebase::ts_source::FileInventory;

#[test]
fn missing_facts_use_the_file_checkers() {
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let config = NoMistakesConfig::default();
    let files: &[PathBuf] = &[];
    for rule_id in [
        POSTGRES_CONSTRAINT_VALIDATE,
        POSTGRES_NO_ADD_COLUMN,
        POSTGRES_FK_INDEX,
        POSTGRES_REDUNDANT_INDEX,
        POSTGRES_IDENTIFIER_LENGTH,
        POSTGRES_REQUIRE_FK_ON_DELETE,
        POSTGRES_REQUIRE_NAMED_CONSTRAINTS,
        POSTGRES_SQL_STATEMENT_POLICY,
        DUPLICATE_RULE,
        COMMENTS_RULE,
        COLUMN_RULE,
        CONFLICT_RULE,
        TABLE_RULE,
        STATUS_RULE,
        NAMING_RULE,
        COLUMN_NAMING_RULE,
        IDENTIFIER_LENGTH_RULE,
        FINITE_TEXT_RULE,
        ARRAY_COLUMNS_RULE,
        REQUIRED_PREDICATES_RULE,
        EXPLICIT_COLUMNS_RULE,
        NO_OFFSET_RULE,
    ] {
        assert!(run(rule_id, Path::new("."), &config, files, &sources, None).is_some());
    }
}

#[test]
fn prepared_facts_use_the_fact_checkers() {
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let config = NoMistakesConfig::default();
    let files: &[PathBuf] = &[];
    let facts = CheckFactMap::default();
    for rule_id in [
        POSTGRES_CONSTRAINT_VALIDATE,
        POSTGRES_NO_ADD_COLUMN,
        POSTGRES_FK_INDEX,
        POSTGRES_REDUNDANT_INDEX,
        POSTGRES_IDENTIFIER_LENGTH,
        POSTGRES_REQUIRE_FK_ON_DELETE,
        POSTGRES_REQUIRE_NAMED_CONSTRAINTS,
        POSTGRES_SQL_STATEMENT_POLICY,
        NAMING_RULE,
        COLUMN_NAMING_RULE,
        FINITE_TEXT_RULE,
        ARRAY_COLUMNS_RULE,
        REQUIRED_PREDICATES_RULE,
        EXPLICIT_COLUMNS_RULE,
        NO_OFFSET_RULE,
        TABLE_RULE,
        STATUS_RULE,
    ] {
        assert!(run(
            rule_id,
            Path::new("."),
            &config,
            files,
            &sources,
            Some(&facts)
        )
        .is_some());
    }
}
