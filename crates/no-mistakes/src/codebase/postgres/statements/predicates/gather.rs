use super::*;
pub(super) struct Gather<'a> {
    pub(super) instances: Vec<Instance<'a>>,
    pub(super) from_items: usize,
    pub(super) all_base: bool,
}

impl Default for Gather<'_> {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
            from_items: 0,
            all_base: true,
        }
    }
}

pub(super) fn walk_relation<'a>(
    factor: &'a TableFactor,
    joined: bool,
    on: Option<&'a Expr>,
    ctes: &[String],
    gathered: &mut Gather<'a>,
    projection: &impl Projection,
) {
    match factor {
        TableFactor::Table {
            name, alias, args, ..
        } => {
            gathered.from_items += 1;
            if args.is_some() {
                gathered.all_base = false;
                return;
            }
            let Some(table) = base_table(name, ctes) else {
                gathered.all_base = false;
                return;
            };
            gathered.instances.push(Instance {
                table: projection.relation_name(name, table),
                alias: alias.as_ref().map(|alias| ident_key(&alias.name)),
                line: name.span().start.line as usize,
                extra_on: Vec::new(),
                joined,
                on,
            });
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            projection.nested_join();
            walk_relation(
                &table_with_joins.relation,
                joined,
                on,
                ctes,
                gathered,
                projection,
            );
            for join in &table_with_joins.joins {
                walk_relation(
                    &join.relation,
                    true,
                    super::super::select::join_expr(&join.join_operator),
                    ctes,
                    gathered,
                    projection,
                );
            }
        }
        _ => {
            gathered.from_items += 1;
            gathered.all_base = false;
        }
    }
}
