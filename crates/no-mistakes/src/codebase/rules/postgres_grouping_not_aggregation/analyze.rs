mod lineage;
use super::scope::definitions;
use super::{candidate, existence, filters, union, unique};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::*;
use std::collections::BTreeMap;
use std::ops::ControlFlow;
#[derive(Clone)]
pub(super) struct Finding {
    pub site: (usize, usize),
    pub code: &'static str,
    pub message: &'static str,
}
pub(super) fn finding(select: &Select, code: &'static str, message: &'static str) -> Finding {
    let location = select.select_token.0.span.start;
    Finding {
        site: (location.line as usize, location.column as usize),
        code,
        message,
    }
}
pub(super) fn query(
    query: &Query,
    catalog: &SchemaCatalog,
    parameters: &[(u32, u32)],
) -> Vec<Finding> {
    struct Scan<'a> {
        catalog: &'a SchemaCatalog,
        parameters: &'a [(u32, u32)],
        recursive: Vec<bool>,
        shadowed: Vec<std::collections::BTreeSet<String>>,
        ctes: Vec<BTreeMap<String, union::Columns>>,
        definition_names: definitions::Names,
        definition_ctes: lineage::Overrides,
        order_owners: Vec<std::collections::BTreeSet<String>>,
        out: Vec<Finding>,
    }
    impl Visitor for Scan<'_> {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            let recursive = self.recursive.last().copied().unwrap_or(false)
                || query.with.as_ref().is_some_and(|with| with.recursive);
            self.recursive.push(recursive);
            let shadowed =
                definitions::enter(query, self.shadowed.last(), &mut self.definition_names);
            self.shadowed.push(shadowed);
            let inherited_ctes = self
                .definition_ctes
                .remove(&definitions::key(query))
                .unwrap_or_else(|| self.ctes.last().cloned().unwrap_or_default());
            let scope = super::scope::CatalogScope {
                catalog: self.catalog,
                parameters: self.parameters,
                shadowed: self.shadowed.last().unwrap(),
            };
            let ctes = lineage::projections(
                query,
                &scope,
                recursive,
                inherited_ctes,
                &self.definition_names,
                &mut self.definition_ctes,
            );
            if !recursive {
                union::arms(
                    &query.body,
                    &super::scope::CatalogScope {
                        catalog: self.catalog,
                        parameters: self.parameters,
                        shadowed: self.shadowed.last().unwrap(),
                    },
                    &mut self.out,
                );
                let scope = super::scope::CatalogScope {
                    catalog: self.catalog,
                    parameters: self.parameters,
                    shadowed: self.shadowed.last().unwrap(),
                };
                filters::cte_consumers(query, &scope, &self.definition_names, &mut self.out);
            }
            let mut owners = std::collections::BTreeSet::new();
            let _: ControlFlow<()> = visit_expressions(&query.order_by, |expr| {
                if let Some((owner, name)) = super::expressions::column(expr) {
                    owners.insert(owner.unwrap_or_else(|| format!(":{name}")));
                }
                ControlFlow::Continue(())
            });
            self.order_owners.push(owners);
            self.ctes.push(ctes);
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.recursive.pop();
            self.shadowed.pop();
            self.ctes.pop();
            self.order_owners.pop();
            ControlFlow::Continue(())
        }
        fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
            let scope = super::scope::CatalogScope {
                catalog: self.catalog,
                parameters: self.parameters,
                shadowed: self.shadowed.last().unwrap(),
            };
            if unique::redundant(select, &scope) {
                let message = if select.distinct.is_some()
                    && matches!(&select.group_by, GroupByExpr::Expressions(exprs, _) if exprs.iter().any(|expr| {
                        matches!(expr, Expr::GroupingSets(_) | Expr::Rollup(_) | Expr::Cube(_))
                    })) {
                    "catalog uniqueness proves the projected DISTINCT key is unique across grouping sets; remove DISTINCT and preserve GROUP BY and its subtotal rows"
                } else {
                    "catalog uniqueness and equality pins already identify each deduplication key; remove DISTINCT while preserving COUNT(expression), or remove grouping without aggregates"
                };
                self.out
                    .push(finding(select, "distinct-over-unique-key", message));
            }
            if self.recursive.last() == Some(&true) {
                return ControlFlow::Continue(());
            }
            if candidate::matches(select, &scope) {
                self.out.push(finding(select, "all-of-candidate-pool", "GROUP BY counts candidate matches rather than aggregating a result; use one ANDed EXISTS predicate per required term"));
            }
            if union::outer(select, self.ctes.last().unwrap(), &scope) {
                self.out.push(finding(
                    select,
                    "distinct-over-union",
                    "UNION already deduplicates the same projection; remove the redundant DISTINCT",
                ));
            }
            filters::select_filters(
                select,
                &scope,
                self.order_owners.last().unwrap(),
                &mut self.out,
            );
            existence::select_tests(select, &scope, &mut self.out);
            ControlFlow::Continue(())
        }
    }
    let mut scan = Scan {
        catalog,
        parameters,
        recursive: Vec::new(),
        shadowed: Vec::new(),
        ctes: Vec::new(),
        definition_names: Default::default(),
        definition_ctes: Default::default(),
        order_owners: Vec::new(),
        out: Vec::new(),
    };
    let _ = query.visit(&mut scan);
    scan.out.sort_by_key(|finding| (finding.site, finding.code));
    scan.out
        .dedup_by_key(|finding| (finding.site, finding.code));
    scan.out
}
