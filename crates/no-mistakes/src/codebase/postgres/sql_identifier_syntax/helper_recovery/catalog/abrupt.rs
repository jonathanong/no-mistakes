//! An abrupt return proof must not hide source-visible selected SQL effects.
use super::{Key, PreparedCatalog};
use crate::codebase::postgres::sql_identifier_syntax::Span;
use std::path::Path;

impl PreparedCatalog<'_> {
    fn selected_sql(&self, file: &Path, at: Option<Span>) -> bool {
        let Some(facts) = self.files.get(file) else {
            return false;
        };
        // A companion can collect this profile outside the requesting application's roots.
        if self.application.is_some_and(|application| {
            !facts
                .sql_identifier_syntax
                .includes_application(application)
        }) {
            return false;
        }
        facts
            .sql_identifier_syntax
            .profiles
            .iter()
            .filter(|(profile, _)| self.profile.is_none_or(|wanted| profile == wanted))
            .flat_map(|(_, calls)| calls)
            .any(|call| at.is_none_or(|(start, end)| start <= call.span.0 && call.span.1 <= end))
    }

    pub(super) fn throw_has_no_selected_sql(
        &self,
        file: &Path,
        at: Span,
        stack: &mut Vec<Key>,
        work: &mut usize,
    ) -> bool {
        let Some(facts) = self.files.get(file) else {
            return false;
        };
        if self.selected_sql(file, Some(at)) {
            return false;
        }
        for call in facts
            .sql_identifier_syntax
            .recovery
            .calls
            .iter()
            .filter(|call| at.0 <= call.span.0 && call.span.1 <= at.1)
        {
            *work += 1;
            if *work > 10_000 {
                return false;
            }
            let Some(id) = call.callee else {
                continue;
            };
            let Some(key) = self.binding(file, id) else {
                // A source callee without a model cannot conceal known SQL facts.
                if self.selected_sql(file, None) {
                    return false;
                }
                let target = self
                    .module(file)
                    .and_then(|module| {
                        module.imports.iter().find(|import| {
                            import
                                .bindings
                                .iter()
                                .any(|binding| binding.binding_id == id)
                        })
                    })
                    .and_then(|import| (self.resolve)(&import.specifier, file));
                if target.is_some_and(|target| self.selected_sql(&target, None)) {
                    return false;
                }
                continue;
            };
            if stack.len() >= 8 || stack.contains(&key) {
                return false;
            }
            let Some(model) = self.files.get(&key.file).and_then(|facts| {
                facts
                    .sql_identifier_syntax
                    .recovery
                    .functions
                    .get(&key.callable)
            }) else {
                return false;
            };
            stack.push(key.clone());
            let clear = self.throw_has_no_selected_sql(&key.file, model.span, stack, work);
            stack.pop();
            if !clear {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
