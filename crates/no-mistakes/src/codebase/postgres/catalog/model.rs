#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Table,
    PartitionedTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedKind {
    Stored,
    Virtual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogTable {
    pub name: String,
    pub relation_kind: RelationKind,
    pub comment: Option<String>,
    pub columns: Vec<CatalogColumn>,
    pub primary_key: Option<Vec<String>>,
    pub foreign_keys: Vec<CatalogForeignKey>,
    pub check_constraints: Vec<CatalogCheck>,
    pub unique_constraints: Vec<CatalogUnique>,
    pub indexes: Vec<CatalogIndexInfo>,
    pub triggers: Vec<CatalogTrigger>,
    pub partition_key: Option<PartitionKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_expression: Option<String>,
    pub generated: Option<GeneratedKind>,
    pub generated_expression: Option<String>,
    pub identity: Option<String>,
    pub comment: Option<String>,
    pub ordinal_position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
    pub validated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCheck {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogUnique {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogIndexInfo {
    pub name: String,
    pub unique: bool,
    pub primary: bool,
    pub constraint_backed: bool,
    pub access_method: String,
    pub keys: Vec<CatalogIndexKey>,
    pub predicate: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogIndexKey {
    pub column: Option<String>,
    pub expression: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
    Truncate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogTrigger {
    pub name: String,
    pub timing: TriggerTiming,
    pub events: Vec<TriggerEvent>,
    pub update_columns: Vec<String>,
    pub for_each_row: bool,
    pub function: String,
    pub arguments: Vec<String>,
    pub when: Option<String>,
    pub definition: String,
}

impl CatalogTrigger {
    /// `events` matches when it is a subset of this trigger's events.
    pub fn matches(
        &self,
        function: &str,
        timing: TriggerTiming,
        events: &[TriggerEvent],
        for_each_row: bool,
    ) -> bool {
        self.function == function
            && self.timing == timing
            && self.for_each_row == for_each_row
            && events.iter().all(|event| self.events.contains(event))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionStrategy {
    Range,
    List,
    Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionKey {
    pub strategy: PartitionStrategy,
    pub elements: Vec<PartitionKeyElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionKeyElement {
    Column(String),
    Expression(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogFunction {
    pub key: String,
    pub name: String,
    pub signature: Option<String>,
    pub language: Option<String>,
    pub returns_trigger: bool,
    pub returns_event_trigger: bool,
    pub definition: String,
    pub body: Option<String>,
    pub body_span: Option<(usize, usize)>,
    pub null_input: String,
    pub security: String,
    pub parallel: String,
    pub leakproof: String,
    pub volatility: String,
    pub return_contract: String,
    pub planner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogView {
    pub name: String,
    pub materialized: bool,
    pub definition: String,
    pub comment: Option<String>,
}
