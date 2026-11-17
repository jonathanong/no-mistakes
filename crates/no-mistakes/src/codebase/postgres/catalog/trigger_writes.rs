use super::{SchemaCatalog, TriggerEvent};
use crate::codebase::postgres::dml::writes::names;
use crate::codebase::postgres::idents::object_name_identity;
use sqlparser::ast::{FromTable, OnConflictAction, OnInsert, Statement, Visit, Visitor};
use std::{collections::BTreeMap, ops::ControlFlow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogTriggerWrites {
    pub source_relation: String,
    pub events: Vec<TriggerEvent>,
    pub update_columns: Vec<String>,
    pub writes: Vec<CatalogTriggerWrite>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogTriggerWrite {
    pub relation: String,
    pub event: TriggerEvent,
    pub columns: Vec<String>,
}

impl SchemaCatalog {
    pub(crate) fn trigger_write_effects(&self) -> &[CatalogTriggerWrites] {
        self.projections().trigger_writes.get_or_init(|| {
            let mut functions: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for function in self.functions().filter(|function| function.returns_trigger) {
                functions
                    .entry(super::names::normalize_table_name(&function.name))
                    .or_default()
                    .push(function);
            }
            let mut parsed_writes = BTreeMap::new();
            let mut out = Vec::new();
            for table in self.tables() {
                for trigger in &table.triggers {
                    if trigger.when.is_some() {
                        continue;
                    }
                    let identity = &self.trigger_function_identities[&table.name][&trigger.name];
                    let selected = functions.get(identity).or_else(|| {
                        let (schema, bare) = super::names::split_key(identity);
                        if schema.as_deref()? != self.declared_schema()? {
                            return None;
                        }
                        let bare = format!("\"{}\"", bare.replace('"', "\"\""));
                        functions.get(&super::names::normalize_table_name(&bare))
                    });
                    let Some(functions) = selected else {
                        continue;
                    };
                    let [function] = functions.as_slice() else {
                        continue;
                    };
                    let Some(writes) = parsed_writes.entry(&function.key).or_insert_with(|| {
                        function_writes(function.body.as_deref()?, function.language.as_deref())
                    }) else {
                        continue;
                    };
                    if !writes.is_empty() {
                        out.push(CatalogTriggerWrites {
                            source_relation: table.name.clone(),
                            events: trigger.events.clone(),
                            update_columns: trigger.update_columns.clone(),
                            writes: writes.clone(),
                        });
                    }
                }
            }
            out
        })
    }
}

fn function_writes(body: &str, language: Option<&str>) -> Option<Vec<CatalogTriggerWrite>> {
    let sql = match language? {
        "sql" => body,
        "plpgsql" => procedural_sql(body)?,
        _ => return None,
    };
    let statements = crate::codebase::postgres::parse::parse_postgres_sql(sql).ok()?;
    if statements.iter().any(|statement| {
        !matches!(
            statement,
            Statement::Insert(_)
                | Statement::Update(_)
                | Statement::Delete(_)
                | Statement::Query(_)
        )
    }) {
        return None;
    }
    let mut collector = Writes(Vec::new());
    for statement in statements {
        let _ = statement.visit(&mut collector);
    }
    Some(collector.0)
}

fn procedural_sql(body: &str) -> Option<&str> {
    let body = body.trim();
    let body = strip_word(body, "begin")?.trim();
    let body = body.trim_end_matches(';').trim_end();
    let end = body.len().checked_sub(3)?;
    if !body.get(end..)?.eq_ignore_ascii_case("end") {
        return None;
    }
    let body = body[..end].trim_end();
    let previous = body.trim_end_matches(';').trim_end();
    let start = previous.rfind(';').map_or(0, |index| index + 1);
    let tail = previous[start..].trim();
    let value = strip_word(tail, "return")?.trim();
    if !["new", "old", "null"]
        .iter()
        .any(|item| value.eq_ignore_ascii_case(item))
    {
        return None;
    }
    Some(&previous[..start])
}

fn strip_word<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    super::function_body::is_word_at(text, 0, word).then(|| &text[word.len()..])
}

struct Writes(Vec<CatalogTriggerWrite>);
impl Writes {
    fn push(&mut self, relation: String, event: TriggerEvent, columns: Vec<String>) {
        self.0.push(CatalogTriggerWrite {
            relation,
            event,
            columns,
        });
    }
}
impl Visitor for Writes {
    type Break = ();
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        match statement {
            Statement::Insert(insert) => {
                let table = names::table_object_name(&insert.table)
                    .expect("PostgreSQL INSERT target is a table name");
                self.push(table.clone(), TriggerEvent::Insert, Vec::new());
                if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
                    if let OnConflictAction::DoUpdate(update) = &conflict.action {
                        self.push(table, TriggerEvent::Update, columns(&update.assignments));
                    }
                }
            }
            Statement::Update(update) => {
                if let Some(table) = names::table_with_joins_name(&update.table) {
                    self.push(table, TriggerEvent::Update, columns(&update.assignments));
                }
            }
            Statement::Delete(delete) => {
                let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) =
                    &delete.from;
                for table in from {
                    if let sqlparser::ast::TableFactor::Table { name, .. } = &table.relation {
                        self.push(object_name_identity(name), TriggerEvent::Delete, Vec::new());
                    }
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
fn columns(assignments: &[sqlparser::ast::Assignment]) -> Vec<String> {
    assignments
        .iter()
        .flat_map(|value| names::assignment_base_column_keys(&value.target))
        .collect()
}
