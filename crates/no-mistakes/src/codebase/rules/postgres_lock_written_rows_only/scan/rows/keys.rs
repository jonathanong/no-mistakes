use super::{EmbeddedSqlCall, RowAccess};

pub(in super::super) fn compare_keys(
    lock: &RowAccess,
    lock_call: &EmbeddedSqlCall,
    write: &RowAccess,
    write_call: &EmbeddedSqlCall,
) -> Option<bool> {
    if lock.key_identity_unproven || write.key_identity_unproven {
        return None;
    }
    if std::ptr::eq(lock_call, write_call) && lock_call.caller_transaction_id.is_none() {
        let (lock_column, lock_key) = lock.key.as_ref()?;
        let (write_column, write_key) = write.key.as_ref()?;
        if lock_key.starts_with([':', '?']) || write_key.starts_with([':', '?']) {
            return None;
        }
        return (lock_column == write_column).then_some(lock_key == write_key);
    }
    let (lock_column, lock_key) = proven_key(lock, lock_call)?;
    let (write_column, write_key) = proven_key(write, write_call)?;
    (lock_column == write_column).then_some(lock_key == write_key)
}

pub(in super::super) fn proven_key<'a>(
    row: &'a RowAccess,
    call: &EmbeddedSqlCall,
) -> Option<(&'a str, String)> {
    if row.key_identity_unproven {
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

pub(in super::super) fn stable_prelock_key(row: &RowAccess, call: &EmbeddedSqlCall) -> bool {
    row.key.as_ref().is_some_and(|(_, value)| {
        if value.starts_with('$') {
            parameter_index(value)
                .and_then(|index| call.stable_prelock_parameters.get(index))
                .copied()
                == Some(true)
        } else {
            !value.starts_with([':', '?'])
        }
    })
}

fn parameter_index(value: &str) -> Option<usize> {
    value
        .strip_prefix('$')?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
}

pub(super) fn identity(value: &str, call: &EmbeddedSqlCall) -> Option<String> {
    if value.starts_with([':', '?']) {
        return None;
    }
    if let Some(index) = parameter_index(value) {
        return call.bound_parameters.get(index)?.clone();
    }
    Some(format!("literal:{value}"))
}
