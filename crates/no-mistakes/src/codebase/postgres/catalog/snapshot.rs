use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Snapshot {
    /// Stated by the generator, never assumed: a missing value is a load error.
    pub(super) coverage: super::CatalogCoverage,
    #[serde(default)]
    pub(super) schema: Option<String>,
    /// Only explicitly requested schemas are authoritative; a missing key is unknown.
    #[serde(default)]
    pub(super) search_path_evidence: BTreeMap<String, Option<BTreeSet<String>>>,
    #[serde(default)]
    pub(super) tables: BTreeMap<String, SnapshotTable>,
    #[serde(default)]
    pub(super) functions: BTreeMap<String, SnapshotFunction>,
    #[serde(default)]
    pub(super) enums: BTreeMap<String, SnapshotEnum>,
    #[serde(default)]
    pub(super) views: BTreeMap<String, SnapshotView>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotTable {
    pub(super) relation_kind: String,
    pub(super) comment: Option<String>,
    pub(super) columns: BTreeMap<String, SnapshotColumn>,
    pub(super) primary_key: Option<SnapshotPrimaryKey>,
    pub(super) foreign_keys: BTreeMap<String, SnapshotForeignKey>,
    pub(super) check_constraints: BTreeMap<String, SnapshotCheck>,
    pub(super) unique_constraints: BTreeMap<String, SnapshotKeyConstraint>,
    pub(super) indexes: BTreeMap<String, SnapshotIndex>,
    pub(super) triggers: BTreeMap<String, SnapshotTrigger>,
    pub(super) physical_partition: Option<SnapshotPartition>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SnapshotColumn {
    /// Required: a missing type must fail the load rather than become `""`.
    pub(super) data_type: String,
    #[serde(default)]
    pub(super) nullable: bool,
    #[serde(default)]
    pub(super) default_expression: Option<String>,
    #[serde(default)]
    pub(super) generated: Option<String>,
    #[serde(default)]
    pub(super) generated_expression: Option<String>,
    #[serde(default)]
    pub(super) identity: Option<String>,
    #[serde(default)]
    pub(super) comment: Option<String>,
    #[serde(default)]
    pub(super) ordinal_position: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotPrimaryKey {
    pub(super) columns: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotForeignKey {
    pub(super) columns: Vec<String>,
    pub(super) referenced_table: String,
    pub(super) referenced_columns: Vec<String>,
    pub(super) on_delete: String,
    pub(super) on_update: String,
    pub(super) validated: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotCheck {
    pub(super) definition: String,
    #[serde(default = "default_validated")]
    pub(super) validated: bool,
}

fn default_validated() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotKeyConstraint {
    pub(super) columns: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotIndex {
    pub(super) immediate: Option<bool>,
    pub(super) live: Option<bool>,
    pub(super) access_method: String,
    pub(super) unique: bool,
    pub(super) primary: bool,
    pub(super) constraint_backed: bool,
    pub(super) valid: bool,
    pub(super) ready: bool,
    pub(super) keys: Vec<SnapshotIndexKey>,
    pub(super) predicate: Option<String>,
    pub(super) definition: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotIndexKey {
    pub(super) ordering_supported: Option<bool>,
    pub(super) column: Option<String>,
    pub(super) expression: String,
    pub(super) descending: bool,
    pub(super) nulls_first: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotTrigger {
    pub(super) definition: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotPartition {
    pub(super) key: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotFunction {
    pub(super) definition: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotEnum {
    pub(super) values: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct SnapshotView {
    pub(super) materialized: bool,
    pub(super) definition: String,
    pub(super) comment: Option<String>,
}
