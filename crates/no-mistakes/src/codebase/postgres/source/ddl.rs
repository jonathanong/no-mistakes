//! Typed DDL projections borrow the request AST and location index.
use super::{expressions, locations::Locations, type_facts, types::*};
use crate::codebase::postgres::statements::TableTokenIndex;
use sqlparser::ast::{
    CreateFunction, CreateFunctionBody, CreateTrigger, CreateView, FunctionReturnType,
};
mod dependencies;
mod trigger_tokens;
pub(super) use trigger_tokens::prepare as prepare_trigger_arguments;

pub(super) fn view(
    view: &CreateView,
    locations: &Locations<'_>,
    tokens: &TableTokenIndex,
    recursive: bool,
) -> PostgresSqlView {
    let (dependencies, dependencies_complete, functions) =
        dependencies::collect(&view.query, tokens, locations);
    PostgresSqlView {
        name: expressions::name(&view.name),
        columns: view
            .columns
            .iter()
            .map(|column| expressions::identifier(&column.name))
            .collect(),
        materialized: view.materialized,
        temporary: view.temporary,
        or_replace: view.or_replace,
        // Compatibility restoration wraps the original body in an implicit CTE.
        // Source facts retain the declared body; AST consumers use the full scope.
        query: if recursive {
            view.query.with.as_ref().unwrap().cte_tables[0]
                .query
                .to_string()
        } else {
            view.query.to_string()
        },
        dependencies,
        dependencies_complete,
        functions,
    }
}

pub(super) fn trigger(trigger: &CreateTrigger, locations: &Locations<'_>) -> PostgresSqlTrigger {
    PostgresSqlTrigger {
        name: expressions::name(&trigger.name),
        table: expressions::name(&trigger.table_name),
        timing: trigger.period.as_ref().map(ToString::to_string),
        events: trigger.events.iter().map(ToString::to_string).collect(),
        for_each: trigger.trigger_object.as_ref().map(ToString::to_string),
        condition: trigger
            .condition
            .as_ref()
            .map(|expr| expressions::expression(expr, locations)),
        function: trigger
            .exec_body
            .as_ref()
            .map(|body| expressions::name(&body.func_desc.name)),
        arguments: trigger
            .exec_body
            .as_ref()
            .and_then(|body| body.func_desc.args.as_ref())
            .map(|args| args.iter().map(ToString::to_string).collect())
            .unwrap_or_default(),
        constraint: trigger.is_constraint,
        characteristics: trigger.characteristics.as_ref().map(ToString::to_string),
        or_replace: trigger.or_replace,
        referenced_table: trigger
            .referenced_table_name
            .as_ref()
            .map(expressions::name),
        transitions: trigger
            .referencing
            .iter()
            .map(|reference| PostgresSqlTriggerTransition {
                kind: reference.refer_type.to_string(),
                name: expressions::name(&reference.transition_relation_name),
            })
            .collect(),
        execution_kind: trigger
            .exec_body
            .as_ref()
            .map(|body| body.exec_type.to_string()),
    }
}

pub(super) fn function(
    function: &CreateFunction,
    locations: &Locations<'_>,
) -> PostgresSqlFunction {
    let (return_type, returns_set) = match &function.return_type {
        Some(FunctionReturnType::DataType(data_type)) => {
            (Some(type_facts::data_type(data_type, locations)), false)
        }
        Some(FunctionReturnType::SetOf(data_type)) => {
            (Some(type_facts::data_type(data_type, locations)), true)
        }
        None => (None, false),
    };
    PostgresSqlFunction {
        name: expressions::name(&function.name),
        arguments: function
            .args
            .iter()
            .flatten()
            .map(|argument| PostgresSqlFunctionArgument {
                name: argument.name.as_ref().map(expressions::identifier),
                mode: argument.mode.as_ref().map(ToString::to_string),
                data_type: type_facts::data_type(&argument.data_type, locations),
                default: argument
                    .default_expr
                    .as_ref()
                    .map(|expr| expressions::expression(expr, locations)),
            })
            .collect(),
        return_type,
        returns_set,
        language: function
            .language
            .as_ref()
            .map(|language| language.value.clone()),
        behavior: function.behavior.as_ref().map(ToString::to_string),
        security: function.security.as_ref().map(ToString::to_string),
        body_sql: function.function_body.as_ref().map(body_sql),
        or_replace: function.or_replace,
        temporary: function.temporary,
        called_on_null: function.called_on_null.as_ref().map(ToString::to_string),
        parallel: function.parallel.as_ref().map(ToString::to_string),
        configuration: function
            .set_params
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

fn body_sql(body: &CreateFunctionBody) -> String {
    match body {
        CreateFunctionBody::AsBeforeOptions { body, link_symbol } => {
            let mut sql = format!("AS {body}");
            if let Some(symbol) = link_symbol {
                sql.push_str(&format!(", {symbol}"));
            }
            sql
        }
        CreateFunctionBody::AsAfterOptions(body) => format!("AS {body}"),
        CreateFunctionBody::AsBeginEnd(body) => format!("AS {body}"),
        CreateFunctionBody::Return(body) => format!("RETURN {body}"),
        CreateFunctionBody::AsReturnExpr(body) => format!("AS RETURN {body}"),
        CreateFunctionBody::AsReturnSelect(body) => format!("AS RETURN {body}"),
    }
}
