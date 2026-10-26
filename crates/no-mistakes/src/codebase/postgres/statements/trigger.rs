use super::{SqlTriggerEvent, SqlTriggerFact, SqlTriggerPeriod};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    CreateTrigger, DropTrigger, Spanned, Statement, TriggerEvent, TriggerObject, TriggerObjectKind,
    TriggerPeriod,
};

pub(super) fn from_statement(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    statement: &Statement,
) -> Option<SqlTriggerFact> {
    match statement {
        Statement::CreateTrigger(trigger) => Some(from_trigger(tokens, trigger)),
        _ => None,
    }
}

fn from_trigger(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    trigger: &CreateTrigger,
) -> SqlTriggerFact {
    let location = crate::codebase::postgres::migration::statement_keyword_position(
        tokens,
        "CREATE",
        trigger.name.span().start,
    );
    SqlTriggerFact {
        table: relation_name(&trigger.table_name),
        function: trigger
            .exec_body
            .as_ref()
            .map(|body| relation_name(&body.func_desc.name))
            .unwrap_or_default(),
        period: match trigger.period {
            Some(TriggerPeriod::Before) => SqlTriggerPeriod::Before,
            Some(TriggerPeriod::After) => SqlTriggerPeriod::After,
            Some(TriggerPeriod::InsteadOf) => SqlTriggerPeriod::InsteadOf,
            _ => SqlTriggerPeriod::Other,
        },
        for_each_row: matches!(
            trigger.trigger_object,
            Some(TriggerObjectKind::ForEach(TriggerObject::Row))
                | Some(TriggerObjectKind::For(TriggerObject::Row))
        ),
        events: trigger.events.iter().map(from_event).collect(),
        line: location.line.max(1) as usize,
        column: location.column.max(1) as usize,
        name: relation_name(&trigger.name),
        removed: false,
        source_path: std::path::PathBuf::new(),
    }
}

pub(super) fn from_drop(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    drop: &DropTrigger,
) -> SqlTriggerFact {
    let location = crate::codebase::postgres::migration::statement_keyword_position(
        tokens,
        "DROP",
        drop.trigger_name.span().start,
    );
    SqlTriggerFact {
        table: drop
            .table_name
            .as_ref()
            .map(relation_name)
            .unwrap_or_default(),
        function: String::new(),
        period: SqlTriggerPeriod::Other,
        for_each_row: false,
        events: Vec::new(),
        line: location.line.max(1) as usize,
        column: location.column.max(1) as usize,
        name: relation_name(&drop.trigger_name),
        removed: true,
        source_path: std::path::PathBuf::new(),
    }
}

fn from_event(event: &TriggerEvent) -> SqlTriggerEvent {
    match event {
        TriggerEvent::Insert => SqlTriggerEvent::Insert,
        TriggerEvent::Delete => SqlTriggerEvent::Delete,
        TriggerEvent::Truncate => SqlTriggerEvent::Truncate,
        TriggerEvent::Update(columns) => SqlTriggerEvent::Update {
            columns: columns.iter().map(|ident| ident.value.clone()).collect(),
        },
    }
}
