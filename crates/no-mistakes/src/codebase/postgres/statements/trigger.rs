use super::{SqlTriggerEvent, SqlTriggerFact, SqlTriggerPeriod};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    CreateTrigger, Spanned, Statement, TriggerEvent, TriggerObject, TriggerObjectKind,
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
        line: crate::codebase::postgres::migration::statement_keyword_line(
            tokens,
            "CREATE",
            trigger.name.span().start,
        ),
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
