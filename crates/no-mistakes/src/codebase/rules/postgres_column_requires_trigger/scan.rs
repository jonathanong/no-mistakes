use super::{Compiled, Requirement, RULE_ID};
use crate::codebase::postgres::{
    catalog_finding, CatalogObjectRef, CatalogTable, CatalogTrigger, SchemaCatalog, TriggerEvent,
    TriggerTiming,
};
use crate::codebase::rules::RuleFinding;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        for requirement in &compiled.requirements {
            findings.extend(findings_for(
                &compiled.schema_catalog_path,
                table,
                requirement,
            ));
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn findings_for(
    catalog_path: &str,
    table: &CatalogTable,
    requirement: &Requirement,
) -> Vec<RuleFinding> {
    let object = CatalogObjectRef::Table(table.name.clone());
    let has_column = table
        .columns
        .iter()
        .any(|column| column.name == requirement.column);
    if !has_column {
        return table
            .triggers
            .iter()
            .filter(|trigger| same_function(trigger, requirement))
            .map(|trigger| {
                catalog_finding(
                    RULE_ID,
                    catalog_path,
                    &object,
                    &format!(
                        "trigger {} executes {}() but the table has no {} column",
                        trigger.name, requirement.function, requirement.column
                    ),
                )
            })
            .collect();
    }
    if table
        .triggers
        .iter()
        .any(|trigger| covers(trigger, requirement))
    {
        return Vec::new();
    }
    let column_lists: Vec<&CatalogTrigger> = table
        .triggers
        .iter()
        .filter(|trigger| column_list_only(trigger, requirement))
        .collect();
    if let [trigger] = column_lists.as_slice() {
        let columns = trigger.update_columns.join(", ");
        return vec![catalog_finding(
            RULE_ID,
            catalog_path,
            &object,
            &format!(
                "trigger {} executes {}() only for UPDATE OF {columns}; it must fire on every UPDATE",
                trigger.name, requirement.function
            ),
        )];
    }
    vec![catalog_finding(
        RULE_ID,
        catalog_path,
        &object,
        &format!(
            "table has column {} but no {} trigger executing {}()",
            requirement.column,
            requirement_label(requirement),
            requirement.function
        ),
    )]
}

fn covers(trigger: &CatalogTrigger, requirement: &Requirement) -> bool {
    trigger.matches(
        &requirement.function,
        requirement.timing,
        &requirement.events,
        requirement.for_each_row,
    ) && (requirement.allow_column_list
        || !requirement.events.contains(&TriggerEvent::Update)
        || trigger.update_columns.is_empty())
}

fn column_list_only(trigger: &CatalogTrigger, requirement: &Requirement) -> bool {
    trigger.matches(
        &requirement.function,
        requirement.timing,
        &requirement.events,
        requirement.for_each_row,
    ) && !requirement.allow_column_list
        && requirement.events.contains(&TriggerEvent::Update)
        && !trigger.update_columns.is_empty()
}

fn same_function(trigger: &CatalogTrigger, requirement: &Requirement) -> bool {
    trigger.function == requirement.function
}

fn requirement_label(requirement: &Requirement) -> String {
    let timing = match requirement.timing {
        TriggerTiming::Before => "BEFORE",
        TriggerTiming::After => "AFTER",
        TriggerTiming::InsteadOf => "INSTEAD OF",
    };
    let events = requirement
        .events
        .iter()
        .map(|event| match event {
            TriggerEvent::Insert => "INSERT",
            TriggerEvent::Update => "UPDATE",
            TriggerEvent::Delete => "DELETE",
            TriggerEvent::Truncate => "TRUNCATE",
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let each = if requirement.for_each_row {
        "FOR EACH ROW"
    } else {
        "FOR EACH STATEMENT"
    };
    format!("{timing} {events} {each}")
}
