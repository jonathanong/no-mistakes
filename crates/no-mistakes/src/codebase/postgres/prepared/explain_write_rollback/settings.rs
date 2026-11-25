//! Read-only transaction and session setting recognition for EXPLAIN write rollback.
use super::literals::{parse_bool_guc, string_literal, three_arg_exprs, unwrap_expr};
use crate::codebase::postgres::idents::object_name_identity;
use sqlparser::ast::{
    ContextModifier, Expr, ObjectNamePart, Query, SelectItem, Set, SetExpr, TransactionAccessMode,
    TransactionMode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadOnlySetting {
    DefaultTransaction,
    CurrentTransaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingEffect {
    Change {
        setting: ReadOnlySetting,
        read_only: bool,
        is_local: bool,
    },
    Unanalyzable(&'static str),
}

pub(super) fn start_transaction_can_commit(
    modes: &[TransactionMode],
    session_read_only: bool,
) -> bool {
    let mut can_commit = !session_read_only;
    for mode in modes {
        match mode {
            TransactionMode::AccessMode(TransactionAccessMode::ReadOnly) => can_commit = false,
            TransactionMode::AccessMode(TransactionAccessMode::ReadWrite) => can_commit = true,
            _ => {}
        }
    }
    can_commit
}

pub(super) fn set_statement_effect(set: &Set) -> Option<SettingEffect> {
    match set {
        Set::SetTransaction { modes, session, .. } => {
            let mut access = None;
            for mode in modes {
                if let TransactionMode::AccessMode(mode) = mode {
                    access = Some(matches!(mode, TransactionAccessMode::ReadOnly));
                }
            }
            access.map(|read_only| SettingEffect::Change {
                setting: if *session {
                    ReadOnlySetting::DefaultTransaction
                } else {
                    ReadOnlySetting::CurrentTransaction
                },
                read_only,
                is_local: false,
            })
        }
        Set::SingleAssignment {
            scope,
            variable,
            values,
            ..
        } => {
            let [ObjectNamePart::Identifier(name)] = variable.0.as_slice() else {
                return None;
            };
            let setting = read_only_setting(&name.value)?;
            let is_local = matches!(scope, Some(ContextModifier::Local));
            let read_only = match values.as_slice() {
                [value] => parse_bool_guc(value),
                _ => None,
            };
            Some(match read_only {
                Some(read_only) => SettingEffect::Change {
                    setting,
                    read_only,
                    is_local,
                },
                None => SettingEffect::Unanalyzable(
                    "SET read-only setting value could not be statically evaluated",
                ),
            })
        }
        _ => None,
    }
}

pub(super) fn query_set_config_effects(query: &Query) -> Vec<SettingEffect> {
    if query.with.is_some() || query.limit_clause.is_some() {
        return Vec::new();
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Vec::new();
    };
    if !select.from.is_empty() || select.selection.is_some() {
        return Vec::new();
    }
    let mut effects = Vec::new();
    for item in &select.projection {
        let expr = match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => expr,
            _ => continue,
        };
        if let Some(effect) = set_config_effect(expr) {
            effects.push(effect);
        }
    }
    effects
}

fn set_config_effect(expr: &Expr) -> Option<SettingEffect> {
    let Expr::Function(function) = unwrap_expr(expr) else {
        return None;
    };
    if !matches!(
        object_name_identity(&function.name).as_str(),
        "set_config" | "pg_catalog.set_config"
    ) {
        return None;
    }
    let Some((name_expr, value_expr, local_expr)) = three_arg_exprs(&function.args) else {
        return Some(SettingEffect::Unanalyzable(
            "set_config arguments could not be statically evaluated",
        ));
    };
    let Some(name) = string_literal(name_expr) else {
        return Some(SettingEffect::Unanalyzable(
            "set_config setting name could not be statically evaluated",
        ));
    };
    let setting = read_only_setting(name)?;
    let (Some(read_only), Some(is_local)) =
        (parse_bool_guc(value_expr), parse_bool_guc(local_expr))
    else {
        return Some(SettingEffect::Unanalyzable(
            "set_config read-only arguments could not be statically evaluated",
        ));
    };
    Some(SettingEffect::Change {
        setting,
        read_only,
        is_local,
    })
}

fn read_only_setting(name: &str) -> Option<ReadOnlySetting> {
    if name.eq_ignore_ascii_case("default_transaction_read_only") {
        Some(ReadOnlySetting::DefaultTransaction)
    } else if name.eq_ignore_ascii_case("transaction_read_only") {
        Some(ReadOnlySetting::CurrentTransaction)
    } else {
        None
    }
}
