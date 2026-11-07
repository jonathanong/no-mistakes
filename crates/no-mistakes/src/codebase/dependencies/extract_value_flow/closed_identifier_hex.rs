use super::{literal_string, names::identifier_name, plain_call};
use crate::codebase::dependencies::extract::ImportedBindingKind;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_syntax::operator::BinaryOperator;

impl super::super::super::ImportCollector {
    pub(super) fn generated_hex_name(&self, expr: &Expression<'_>) -> bool {
        let Some(generated) = prefixed_hex(expr) else {
            return false;
        };
        let Some(call) = hex_bytes_call(generated) else {
            return false;
        };
        let (local, member) = match crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(identifier) => (identifier.name.to_string(), false),
            Expression::StaticMemberExpression(member)
                if !member.optional && member.property.name == "randomBytes" =>
            {
                let Some(local) = identifier_name(&member.object) else {
                    return false;
                };
                (local, true)
            }
            _ => return false,
        };
        if self.callee_binding_scope(&local).is_some() {
            return false;
        }
        let mut imports = self
            .call_import_bindings
            .iter()
            .filter(|binding| !binding.is_type_only && binding.local == local);
        let Some(binding) = imports.next() else {
            return false;
        };
        imports.next().is_none()
            && matches!(binding.specifier.as_str(), "node:crypto" | "crypto")
            && if member {
                matches!(
                    binding.kind,
                    ImportedBindingKind::Default | ImportedBindingKind::Namespace
                )
            } else {
                binding.kind == ImportedBindingKind::Named && binding.imported == "randomBytes"
            }
    }
}

fn prefixed_hex<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::TemplateLiteral(template) if template.expressions.len() == 1 => {
            let prefix = crate::ast::template_quasi_text(&template.quasis[0]);
            let suffix = crate::ast::template_quasi_text(&template.quasis[1]);
            (identifier_prefix(&prefix) && identifier_suffix(&suffix))
                .then_some(&template.expressions[0])
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            literal_string(&binary.left)
                .is_some_and(|prefix| identifier_prefix(&prefix))
                .then_some(&binary.right)
        }
        _ => None,
    }
}

fn hex_bytes_call<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    let conversion = plain_call(expr)?;
    let [encoding] = conversion.arguments.as_slice() else {
        return None;
    };
    if literal_string(encoding.as_expression()?).as_deref() != Some("hex") {
        return None;
    }
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&conversion.callee)
    else {
        return None;
    };
    if member.optional || member.property.name != "toString" {
        return None;
    }
    let bytes = plain_call(&member.object)?;
    let [length] = bytes.arguments.as_slice() else {
        return None;
    };
    let Expression::NumericLiteral(length) =
        crate::codebase::ts_source::unwrap_ts_wrappers(length.as_expression()?)
    else {
        return None;
    };
    (length.value >= 0.0 && length.value.fract() == 0.0).then_some(bytes)
}

fn identifier_prefix(text: &str) -> bool {
    text.as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_alphabetic() || *first == b'_')
        && identifier_suffix(text)
}

fn identifier_suffix(text: &str) -> bool {
    text.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

#[cfg(test)]
#[path = "closed_identifier_hex_tests.rs"]
mod tests;
