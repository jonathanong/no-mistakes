use super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::CreateIndex;

pub(super) fn index(value: &CreateIndex, locations: &Locations<'_>) -> PostgresSqlIndex {
    let mut result = PostgresSqlIndex {
        name: value.name.as_ref().map(name),
        table: name(&value.table_name),
        method: value.using.as_ref().map_or_else(
            || "btree".into(),
            |method| method.to_string().to_ascii_lowercase(),
        ),
        unique: value.unique,
        nulls_distinct: value.nulls_distinct.unwrap_or(true),
        keys: value
            .columns
            .iter()
            .map(|key| {
                let ascending = !matches!(
                    key.column.options.sort,
                    Some(sqlparser::ast::OrderBySort::Desc)
                );
                PostgresSqlIndexKey {
                    expression: expression(&key.column.expr, locations),
                    ascending,
                    nulls_first: key.column.options.nulls_first.unwrap_or(!ascending),
                    operator_class: key.operator_class.as_ref().map(name),
                }
            })
            .collect(),
        include: value.include.iter().map(identifier).collect(),
        predicate: value
            .predicate
            .as_ref()
            .map(|expr| expression(expr, locations)),
        options: value
            .with
            .iter()
            .map(|expr| expression(expr, locations).identity)
            .chain(value.index_options.iter().map(ToString::to_string))
            .collect(),
        structural_identity: String::new(),
    };
    // Names and source positions do not define an index's structure. Ordered keys,
    // literal values, quoting, predicates, operator classes and options do.
    let names = |name: &PostgresSqlName| {
        name.parts
            .iter()
            .map(|part| (part.identity.clone(), part.quoted))
            .collect::<Vec<_>>()
    };
    result.structural_identity = serde_json::to_string(&serde_json::json!({
        "table": names(&result.table), "method":result.method, "unique":result.unique,
        "nullsDistinct":result.nulls_distinct,
        "keys":result.keys.iter().map(|key| serde_json::json!([key.expression.identity,key.ascending,key.nulls_first,key.operator_class.as_ref().map(names)])).collect::<Vec<_>>(),
        "include":result.include.iter().map(|part| (&part.identity,part.quoted)).collect::<Vec<_>>(),
        "predicate":result.predicate.as_ref().map(|expr| &expr.identity), "options":result.options
    })).expect("serializable index identity");
    result
}
