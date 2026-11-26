use super::extract::{literal_glob, property, VitestImports};
use super::{spreads, ExcludeArray};
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression};
use oxc_semantic::SymbolId;
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

impl VitestImports<'_, '_> {
    pub(super) fn exclude_array(
        &self,
        test: &ObjectExpression<'_>,
        source: &str,
    ) -> Option<ExcludeArray> {
        let exclude = property(test, "exclude")?;
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(&exclude.value) else {
            return None;
        };
        let mut globs = FxHashSet::default();
        let mut spreads_config_defaults = false;
        let mut imports = Vec::new();
        let mut unknown = false;
        let mut has_spread = false;
        for element in &array.elements {
            if let ArrayExpressionElement::SpreadElement(spread) = element {
                has_spread = true;
                if self.default_spread(&spread.argument) {
                    spreads_config_defaults = true;
                    continue;
                }
                match spreads::read(&spread.argument, self.semantic, &self.relative) {
                    spreads::Spread::Literal(items) => globs.extend(items),
                    spreads::Spread::Imported(import) => imports.push(import),
                    spreads::Spread::Unknown => unknown = true,
                }
            } else if let Some(glob) = element.as_expression().and_then(literal_glob) {
                globs.insert(glob.into_owned());
            } else {
                unknown = true;
            }
        }
        (has_spread || !unknown).then(|| ExcludeArray {
            line: line_number(source, exclude.key.span().start),
            globs,
            spreads_config_defaults,
            imports,
            unknown,
        })
    }

    pub(super) fn is_import(
        &self,
        binding: &oxc_ast::ast::IdentifierReference<'_>,
        imports: &FxHashSet<SymbolId>,
    ) -> bool {
        binding
            .reference_id
            .get()
            .and_then(|id| self.semantic.scoping().get_reference(id).symbol_id())
            .is_some_and(|symbol| imports.contains(&symbol))
    }

    fn default_spread(&self, expression: &Expression<'_>) -> bool {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expression) else {
            return false;
        };
        !member.optional
            && member.property.name == "exclude"
            && matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(binding)
                if self.is_import(binding, &self.defaults))
    }
}
