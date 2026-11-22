use super::{EmbeddedSqlCall, RowAccess};

pub(super) fn compare_keys(
    lock: &RowAccess,
    lock_call: &EmbeddedSqlCall,
    write: &RowAccess,
    write_call: &EmbeddedSqlCall,
) -> Option<bool> {
    let (lock_column, lock_key) = proven_key(lock, lock_call)?;
    let (write_column, write_key) = proven_key(write, write_call)?;
    (lock_column == write_column).then_some(lock_key == write_key)
}

fn proven_key<'a>(row: &'a RowAccess, call: &EmbeddedSqlCall) -> Option<(&'a str, String)> {
    if row.key_aliases_renamed {
        return None;
    }
    if let (Some((column, at)), Some(_)) = (&row.recovered_key, call.caller_transaction_id) {
        if call.bound_value_sql != call.sql_text {
            return None;
        }
        let id = call
            .bound_value_origins
            .iter()
            .find_map(|&(line, column, id)| ((line, column) == *at).then_some(id))?;
        return Some((column, format!("binding:{id}")));
    }
    let (column, value) = row.key.as_ref()?;
    Some((column, identity(value, call)?))
}

pub(super) fn identity(value: &str, call: &EmbeddedSqlCall) -> Option<String> {
    if value.starts_with([':', '?']) {
        return None;
    }
    if let Some(index) = value
        .strip_prefix('$')
        .and_then(|value| value.parse::<usize>().ok())
        .and_then(|index| index.checked_sub(1))
    {
        return call.bound_parameters.get(index)?.clone();
    }
    Some(format!("literal:{value}"))
}
