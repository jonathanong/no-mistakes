//! PostgreSQL recursive views are an implicit WITH RECURSIVE query. Build that
//! AST around the parsed body without reading, tokenizing or parsing again.
use sqlparser::ast::{
    helpers::attached_token::AttachedToken, CreateView, Cte, Expr, GroupByExpr, ObjectName, Query,
    Select, SelectFlavor, SelectItem, SetExpr, TableAlias, TableAliasColumnDef, TableFactor,
    TableWithJoins, With,
};

pub(super) fn wrap(view: &mut CreateView) {
    let name = crate::codebase::postgres::idents::object_name_ident(&view.name)
        .expect("parsed view name has an identifier")
        .clone();
    let columns = view
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let outer = Box::new(Query {
        with: None,
        body: Box::new(SetExpr::Select(Box::new(Select {
            select_token: AttachedToken::empty(),
            optimizer_hints: vec![],
            distinct: None,
            select_modifiers: None,
            top: None,
            top_before_distinct: false,
            projection: columns
                .iter()
                .cloned()
                .map(|name| SelectItem::UnnamedExpr(Expr::Identifier(name)))
                .collect(),
            exclude: None,
            into: None,
            from: vec![TableWithJoins {
                relation: TableFactor::Table {
                    name: ObjectName::from(vec![name.clone()]),
                    alias: None,
                    args: None,
                    with_hints: vec![],
                    version: None,
                    with_ordinality: false,
                    partitions: vec![],
                    json_path: None,
                    sample: None,
                    index_hints: vec![],
                },
                joins: vec![],
            }],
            lateral_views: vec![],
            prewhere: None,
            selection: None,
            connect_by: vec![],
            group_by: GroupByExpr::Expressions(vec![], vec![]),
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            named_window: vec![],
            qualify: None,
            window_before_qualify: false,
            value_table_mode: None,
            flavor: SelectFlavor::Standard,
        }))),
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: vec![],
        for_clause: None,
        settings: None,
        format_clause: None,
        pipe_operators: vec![],
    });
    let body = std::mem::replace(&mut view.query, outer);
    view.query.with = Some(With {
        with_token: AttachedToken::empty(),
        recursive: true,
        cte_tables: vec![Cte {
            alias: TableAlias {
                explicit: false,
                name,
                columns: columns
                    .into_iter()
                    .map(|name| TableAliasColumnDef {
                        name,
                        data_type: None,
                    })
                    .collect(),
                at: None,
            },
            query: body,
            from: None,
            materialized: None,
            closing_paren_token: AttachedToken::empty(),
        }],
    });
}
