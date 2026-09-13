use super::collect_ident_names;
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{SelectItem, SetExpr, Statement};

fn projection_names(sql: &str) -> Vec<String> {
    let Statement::Query(query) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("query");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("select");
    };
    match &select.projection[0] {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            collect_ident_names(expr)
        }
        other => panic!("projection {other:?}"),
    }
}

#[test]
fn collect_ident_names_walks_case_between_in_and_truth_tests() {
    let case_when = projection_names("SELECT CASE flag WHEN true THEN id ELSE note END");
    assert_eq!(case_when, ["flag", "id", "note"]);
    let searched = projection_names("SELECT CASE WHEN flag THEN id ELSE 0 END");
    assert!(searched.contains(&"flag".into()) && searched.contains(&"id".into()));
    assert_eq!(projection_names("SELECT id BETWEEN 1 AND 2"), ["id"]);
    assert_eq!(projection_names("SELECT flag IS TRUE"), ["flag"]);
    assert_eq!(projection_names("SELECT flag IS FALSE"), ["flag"]);
    assert_eq!(projection_names("SELECT id IN (note, 0)"), ["id", "note"]);
    assert_eq!(projection_names("SELECT ((id))"), ["id"]);
    assert_eq!(
        projection_names("SELECT name LIKE pattern"),
        ["name", "pattern"]
    );
    assert_eq!(
        projection_names("SELECT name ILIKE pattern"),
        ["name", "pattern"]
    );
}

#[test]
fn collect_ident_names_walks_named_and_wildcard_function_args() {
    let named = projection_names("SELECT date_trunc('day', timestamp => created_at)");
    assert!(named.contains(&"created_at".into()), "{named:?}");
    assert!(projection_names("SELECT count(*)").is_empty());
}

#[test]
fn ident_helpers_cover_quoted_keys_rlike_and_non_list_function_args() {
    use sqlparser::ast::{
        Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, Ident, ObjectName,
        ObjectNamePart, ObjectNamePartFunction,
    };
    use std::collections::HashSet;

    let quoted = Ident::with_quote('"', "Id");
    assert_eq!(super::ident_key(&quoted), "Id");
    let mut local = HashSet::new();
    super::insert_ident(&mut local, &Ident::new(""));
    assert!(local.is_empty());
    super::insert_ident(&mut local, &quoted);
    assert!(local.contains("Id"));

    let function_name = ObjectName(vec![ObjectNamePart::Function(ObjectNamePartFunction {
        name: Ident::new("fn"),
        args: Vec::new(),
    })]);
    assert!(super::object_name_ident(&function_name).is_none());

    let rlike = Expr::RLike {
        negated: false,
        expr: Box::new(Expr::Identifier(Ident::new("name"))),
        pattern: Box::new(Expr::Identifier(Ident::new("pattern"))),
        regexp: false,
    };
    assert_eq!(collect_ident_names(&rlike), ["name", "pattern"]);

    let none_args = Expr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("now"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    });
    assert!(collect_ident_names(&none_args).is_empty());

    let named_wildcard = FunctionArg::Named {
        name: Ident::new("col"),
        arg: FunctionArgExpr::Wildcard,
        operator: sqlparser::ast::FunctionArgOperator::Equals,
    };
    let mut visited = 0;
    super::visit_function_args(&[named_wildcard], &mut |_| visited += 1);
    assert_eq!(visited, 0);

    let named_expr = FunctionArg::Named {
        name: Ident::new("col"),
        arg: FunctionArgExpr::Expr(Expr::Identifier(Ident::new("created_at"))),
        operator: sqlparser::ast::FunctionArgOperator::Equals,
    };
    let mut names = Vec::new();
    super::visit_function_args(&[named_expr], &mut |expr| {
        names.extend(collect_ident_names(expr));
    });
    assert_eq!(names, ["created_at"]);
}
