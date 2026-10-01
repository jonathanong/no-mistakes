mod cursor;
mod lex;

use super::model::{TriggerEvent, TriggerTiming};
use cursor::Cursor;

#[derive(Debug)]
pub(super) struct ParsedTrigger {
    pub(super) timing: TriggerTiming,
    pub(super) events: Vec<TriggerEvent>,
    pub(super) update_columns: Vec<String>,
    pub(super) for_each_row: bool,
    pub(super) function: String,
    pub(super) arguments: Vec<String>,
    pub(super) when: Option<String>,
}

pub(super) fn parse_trigger(definition: &str) -> Result<ParsedTrigger, String> {
    let mut cursor = Cursor::new(definition);
    cursor.expect_kw("create")?;
    cursor.expect_kw("trigger")?;
    cursor.take_ident()?;
    let timing = cursor.take_timing()?;
    let (events, update_columns) = cursor.take_events()?;
    cursor.expect_kw("on")?;
    cursor.take_qualified()?;
    let mut for_each_row = false;
    if cursor.eat_kw("for") {
        cursor.expect_kw("each")?;
        if cursor.eat_kw("row") {
            for_each_row = true;
        } else {
            cursor.expect_kw("statement")?;
        }
    }
    let when = cursor
        .eat_kw("when")
        .then(|| cursor.take_paren_inner())
        .transpose()?;
    cursor.expect_kw("execute")?;
    if !(cursor.eat_kw("function") || cursor.eat_kw("procedure")) {
        return Err("expected function or procedure".to_string());
    }
    let function = cursor.take_qualified()?;
    let arguments = cursor.take_arguments()?;
    let _ = cursor.eat_char(';');
    cursor.skip_ws();
    if !cursor.done() {
        return Err("trailing trigger syntax".to_string());
    }
    Ok(ParsedTrigger {
        timing,
        events,
        update_columns,
        for_each_row,
        function,
        arguments,
        when,
    })
}
