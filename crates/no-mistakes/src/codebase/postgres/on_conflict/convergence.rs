use crate::codebase::postgres::statement_facts::{SqlOnConflictFact, SqlValueForm};

pub(super) fn judge(
    conflict: &SqlOnConflictFact,
    check_convergence: bool,
    check_volatility: bool,
) -> Option<String> {
    for assignment in &conflict.assignments {
        if let Some(message) = judge_form(
            &assignment.column,
            &assignment.form,
            check_convergence,
            check_volatility,
        ) {
            return Some(message);
        }
    }
    None
}

fn judge_form(
    column: &str,
    form: &SqlValueForm,
    check_convergence: bool,
    check_volatility: bool,
) -> Option<String> {
    match form {
        SqlValueForm::Literal | SqlValueForm::Null | SqlValueForm::Excluded { .. } => None,
        SqlValueForm::SelfRef { column: name } if name.eq_ignore_ascii_case(column) => None,
        SqlValueForm::SelfRef { column: name } if check_convergence => Some(format!(
            "{column} = a reference to {name} is not a self-assignment"
        )),
        SqlValueForm::SelfRef { .. } => None,
        SqlValueForm::Placeholder if check_convergence => Some(format!(
            "{column} = an unresolved bind parameter cannot be proven convergent"
        )),
        SqlValueForm::Placeholder => None,
        SqlValueForm::Subquery if check_convergence => {
            Some(format!("{column} = a subquery cannot be proven convergent"))
        }
        SqlValueForm::Subquery => None,
        SqlValueForm::Volatile { name } if check_volatility => Some(format!(
            "{column} = a volatile value ({name}) re-applies on every replay"
        )),
        SqlValueForm::Volatile { .. } => None,
        SqlValueForm::Coalesce { args } => {
            judge_coalesce(column, args, check_convergence, check_volatility)
        }
        SqlValueForm::Greatest { args } | SqlValueForm::Least { args } => {
            args.iter().find_map(|arg| match arg {
                SqlValueForm::Volatile { name } if check_volatility => Some(format!(
                    "{column} = a volatile value ({name}) inside GREATEST/LEAST is not idempotent"
                )),
                SqlValueForm::SelfRef { column: name } if name.eq_ignore_ascii_case(column) => None,
                SqlValueForm::Literal | SqlValueForm::Null | SqlValueForm::Excluded { .. } => None,
                _ if check_convergence => Some(format!(
                    "{column} = a COALESCE/GREATEST/LEAST expression is not convergent"
                )),
                _ => None,
            })
        }
        SqlValueForm::Other if check_convergence => {
            Some(format!("{column} = a non-convergent expression"))
        }
        SqlValueForm::Other => None,
    }
}

fn judge_coalesce(
    column: &str,
    args: &[SqlValueForm],
    check_convergence: bool,
    check_volatility: bool,
) -> Option<String> {
    let mut self_seen = false;
    for arg in args {
        match arg {
            SqlValueForm::Volatile { name } if check_volatility => {
                if !self_seen {
                    return Some(format!(
                        "{column} = a volatile value ({name}) may only follow the column in COALESCE"
                    ));
                }
            }
            SqlValueForm::SelfRef { column: name } if name.eq_ignore_ascii_case(column) => {
                self_seen = true;
            }
            other => {
                if judge_form(column, other, check_convergence, check_volatility).is_some()
                    && !matches!(other, SqlValueForm::Literal | SqlValueForm::Null)
                {
                    return Some(format!(
                        "{column} = a COALESCE expression referencing a different column"
                    ));
                }
            }
        }
    }
    None
}
