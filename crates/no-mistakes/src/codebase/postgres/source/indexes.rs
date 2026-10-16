use super::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{CreateIndex, IndexType};

pub(super) fn index(
    value: &CreateIndex,
    locations: &Locations<'_>,
    only: bool,
) -> PostgresSqlIndex {
    let mut result = PostgresSqlIndex {
        name: value.name.as_ref().map(name),
        table: name(&value.table_name),
        only,
        method: value.using.as_ref().map_or_else(
            || "btree".into(),
            |method| match method {
                IndexType::Custom(identifier) if identifier.quote_style.is_some() => {
                    identifier.to_string()
                }
                _ => method.to_string().to_ascii_lowercase(),
            },
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
    let identity = IndexIdentity {
        table: names(&result.table),
        method: &result.method,
        unique: result.unique,
        nulls_distinct: result.nulls_distinct,
        keys: result
            .keys
            .iter()
            .map(|key| {
                (
                    key.expression.identity.as_str(),
                    key.ascending,
                    key.nulls_first,
                    key.operator_class.as_ref().map(names),
                )
            })
            .collect(),
        include: result
            .include
            .iter()
            .map(|part| (part.identity.as_str(), part.quoted))
            .collect(),
        predicate: result.predicate.as_ref().map(|expr| expr.identity.as_str()),
        options: &result.options,
        only: result.only.then_some(true),
    };
    result.structural_identity =
        serde_json::to_string(&identity).expect("serializable index identity");
    result
}

// Field order is the legacy public string contract, independent of serde_json's
// optional preserve_order feature. ONLY remains an additive final discriminator.
type IdentityName = Vec<(String, bool)>;
type IdentityKey<'a> = (&'a str, bool, bool, Option<IdentityName>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexIdentity<'a> {
    table: IdentityName,
    method: &'a str,
    unique: bool,
    nulls_distinct: bool,
    keys: Vec<IdentityKey<'a>>,
    include: Vec<(&'a str, bool)>,
    predicate: Option<&'a str>,
    options: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    only: Option<bool>,
}
