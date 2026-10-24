use crate::codebase::postgres::sql_identifier_syntax::Input;
use sqlparser::ast::{ObjectName, ObjectNamePart, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn classify(
    input: &Input,
    rule_id: &str,
    file: &crate::codebase::check_facts::CheckFileFacts,
    application: &crate::codebase::postgres::sql_identifier_syntax::helper_recovery::ApplicationKey,
    span: (u32, u32),
) -> (Vec<String>, bool) {
    let Input::Sql { text, holes, .. } = input else {
        if let Input::Recovered(proof) = input {
            let alternatives = crate::codebase::postgres::sql_identifier_syntax::literal_projection::
                recovered_statements(file)
                .get(&(application.clone(), span));
            let invalid = alternatives
                .into_iter()
                .flat_map(|items| items.iter())
                .any(Result::is_err);
            return (
                Vec::new(),
                !proof.complete() || invalid || alternatives.is_none(),
            );
        }
        return (Vec::new(), true);
    };
    let Ok(statements) = crate::codebase::postgres::parse_postgres_sql(text) else {
        return (Vec::new(), true);
    };
    // Literal marker-looking table names cannot manufacture an interpolation.
    if holes.is_empty() {
        return (Vec::new(), false);
    }
    let mut relations = Relations::default();
    for statement in statements {
        let _ = statement.visit(&mut relations);
    }
    let mut unknown = false;
    let mut sources = std::collections::BTreeSet::new();
    for (placeholder, source) in holes {
        if !relations
            .names
            .iter()
            .any(|name| name.contains(placeholder))
        {
            continue;
        }
        if rule_id == "postgres-no-unbounded-sql-identifier"
            && relations.names.iter().any(|name| name == placeholder)
        {
            if let Some(source) = source {
                sources.insert(source.clone());
                continue;
            }
        }
        unknown = true;
    }
    (sources.into_iter().collect(), unknown)
}

#[derive(Default)]
pub(super) struct Relations {
    pub(super) names: Vec<String>,
}
impl Visitor for Relations {
    type Break = ();
    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<()> {
        self.names
            .extend(relation.0.iter().filter_map(|part| match part {
                ObjectNamePart::Identifier(id) => Some(id.value.clone()),
                _ => None,
            }));
        ControlFlow::Continue(())
    }
}
