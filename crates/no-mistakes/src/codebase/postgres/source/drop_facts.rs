use super::{expressions::name, types::*};
use sqlparser::ast::Statement;

pub(super) fn drop(statement: &Statement) -> Option<PostgresSqlDrop> {
    let mut result = PostgresSqlDrop {
        object_type: String::new(),
        names: Vec::new(),
        table: None,
        signatures: Vec::new(),
        if_exists: false,
        cascade: false,
        restrict: false,
        temporary: false,
    };
    match statement {
        Statement::Drop {
            object_type,
            names,
            table,
            if_exists,
            cascade,
            restrict,
            temporary,
            ..
        } => {
            result.object_type = object_type.to_string();
            result.names = names.iter().map(name).collect();
            result.table = table.as_ref().map(name);
            result.if_exists = *if_exists;
            result.cascade = *cascade;
            result.restrict = *restrict;
            result.temporary = *temporary;
        }
        Statement::DropTrigger(value) => {
            result.object_type = "TRIGGER".into();
            result.names.push(name(&value.trigger_name));
            result.table = value.table_name.as_ref().map(name);
            result.if_exists = value.if_exists;
            if let Some(option) = &value.option {
                result.cascade = option.to_string() == "CASCADE";
                result.restrict = option.to_string() == "RESTRICT";
            }
        }
        Statement::DropFunction(value) => {
            result.object_type = "FUNCTION".into();
            result.names = value
                .func_desc
                .iter()
                .map(|function| name(&function.name))
                .collect();
            result.signatures = value.func_desc.iter().map(ToString::to_string).collect();
            result.if_exists = value.if_exists;
            if let Some(option) = &value.drop_behavior {
                result.cascade = option.to_string() == "CASCADE";
                result.restrict = option.to_string() == "RESTRICT";
            }
        }
        _ => return None,
    }
    Some(result)
}
