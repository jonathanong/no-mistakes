//! Parameters whose annotation closes the domain: `number`, or an element of a
//! literal array catalog (`(typeof CATALOG)[number]`, directly or by alias).
use super::Bindings;
use crate::codebase::dependencies::extract::{
    ValueExpression, ValueFlowFacts, ValueParameter, ValueType,
};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::postgres_sql_cross_file::{
    is_process_env, unique_export, unique_type_export, walk_expression,
};
use std::path::Path;

impl Bindings<'_> {
    pub(super) fn closed_type(&self, file: &Path, parameter: &ValueParameter) -> bool {
        match &parameter.type_reference {
            Some(ValueType::Number) => true,
            Some(shape) => self.catalog_type(file, shape),
            None => false,
        }
    }

    fn catalog_type(&self, file: &Path, shape: &ValueType) -> bool {
        match shape {
            ValueType::Element(catalog) => self.literal_catalog(file, catalog),
            ValueType::Reference(name) => self.alias_catalog(file, name),
            _ => false,
        }
    }

    fn alias_catalog(&self, file: &Path, name: &str) -> bool {
        let Some(ts) = self.facts.get_ts_facts(file) else {
            return false;
        };
        if let Some(shape) = ts.value_flow.type_shapes.get(name) {
            return matches!(shape, ValueType::Element(catalog) if self.literal_catalog(file, catalog));
        }
        let Some(unique) = self
            .graph
            .and_then(|graph| unique_type_export(graph, self.facts, file, name))
        else {
            return false;
        };
        let Some(producer) = self.facts.get_ts_facts(&unique.producer) else {
            return false;
        };
        matches!(
            producer.value_flow.type_shapes.get(&unique.exported),
            Some(ValueType::Element(catalog)) if self.literal_catalog(&unique.producer, catalog)
        )
    }

    fn literal_catalog(&self, file: &Path, name: &str) -> bool {
        let Some(ts) = self.facts.get_ts_facts(file) else {
            return false;
        };
        if literal_array(&ts.value_flow, name) {
            return true;
        }
        let Some(graph) = self.graph else {
            return false;
        };
        unique_export(graph, self.facts, &self.routes, file, name).is_some_and(|unique| {
            unique.producer != file
                && self
                    .facts
                    .get_ts_facts(&unique.producer)
                    .is_some_and(|ts| literal_array(&ts.value_flow, &unique.exported))
        })
    }
}

/// A module-level array literal with no `process.env` value. Cross-file writes
/// that would open it are `postgres-no-mutable-sql-registry` findings.
pub(crate) fn literal_array(flow: &ValueFlowFacts, name: &str) -> bool {
    flow.bindings.iter().any(|binding| {
        binding.owner.is_none()
            && binding.name == name
            && matches!(binding.value, ValueExpression::Array(_))
            && !reads_env(&binding.value)
    })
}

fn reads_env(expr: &ValueExpression) -> bool {
    let mut found = false;
    walk_expression(expr, &mut |value| found |= is_process_env(value));
    found
}
