use super::super::ScopeVisitor;
use super::{bounded, expression, Recovered};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn recover_variants(
        &self,
        expression: &oxc_ast::ast::Expression<'_>,
    ) -> Option<Vec<Recovered>> {
        self.recover_over_paths(|path| expression::recover(self, expression, 8, path))
    }
    /// Alternatives of an executor call whose argument is ``${b.text}`` of a
    /// fully recovered builder (see `template::builder_text`).
    pub(in crate::codebase::postgres::embedded::walk) fn recover_builder_text(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> Option<Vec<Recovered>> {
        let (template, builder) = super::template::builder_text::shape(call)?;
        self.recover_over_paths(|path| {
            super::template::builder_text::recover(self, template, builder, 8, path)
        })
    }
    fn recover_over_paths(
        &self,
        recover: impl Fn(&[(u64, u32)]) -> Option<Vec<Recovered>>,
    ) -> Option<Vec<Recovered>> {
        if self.loop_depth > 0 {
            return None;
        }
        let mut recovered = Vec::new();
        for path in &self.variant_paths {
            let values = recover(path)?;
            recovered.extend(values.into_iter().map(|mut value| {
                for choice in path {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                value
            }));
        }
        bounded(recovered)
    }
    pub(in crate::codebase::postgres::embedded::walk) fn with_variant_paths(
        &self,
        values: Vec<Recovered>,
    ) -> Option<Vec<Recovered>> {
        let paths: Vec<Recovered> = self
            .variant_paths
            .iter()
            .map(|choices| {
                let mut value = Recovered::empty(1);
                value.choices = choices.clone();
                value
            })
            .collect();
        // Choice-only suffixes do not change the physical mapping.
        let mut result = Vec::new();
        for value in values {
            for path in &paths {
                let path: &Recovered = path;
                if value.choices.iter().any(|(id, arm)| {
                    path.choices
                        .iter()
                        .any(|(other, branch)| id == other && arm != branch)
                }) {
                    continue;
                }
                let mut value = value.clone();
                for choice in &path.choices {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                result.push(value);
            }
        }
        bounded(result)
    }
}
