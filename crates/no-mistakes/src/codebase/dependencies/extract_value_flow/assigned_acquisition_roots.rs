use super::*;

impl ImportCollector {
    /// Return the simple mutable-reference classification while acquiring its
    /// RHS receipt, so the immediate caller need not look up the same binding.
    pub(super) fn record_rhs_acquisition(&mut self, value: &ValueExpression) -> bool {
        #[cfg(all(test, feature = "test-instrumentation"))]
        tests::ENTRIES.with(|n| n.set(n.get() + 1));
        match value {
            ValueExpression::Named { name, scope, .. } if !name.contains('.') => {
                let mutable = self.has_mutable_receiver_binding(scope.unwrap_or(0), name);
                if mutable {
                    self.prior_member_mutations
                        .acquire_rhs_reference(scope.unwrap_or(0), name);
                }
                mutable
            }
            ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
                // An unprojected RHS container acquires every contained root.
                for value in fields.values() {
                    self.record_rhs_acquisition(value);
                }
                false
            }
            _ => {
                roots(value, None, &mut |scope, name| {
                    if self.has_mutable_receiver_binding(scope, name) {
                        self.prior_member_mutations
                            .acquire_rhs_reference(scope, name);
                    }
                });
                false
            }
        }
    }
}

/// Only root identities escape acquisition. Keep member selectors borrowed on
/// the stack instead of constructing owned canonical paths that are discarded.
struct MemberPath<'a> {
    head: &'a str,
    tail: Option<&'a MemberPath<'a>>,
}

fn roots(
    value: &ValueExpression,
    path: Option<&MemberPath<'_>>,
    output: &mut impl FnMut(usize, &str),
) {
    match value {
        ValueExpression::Named { name, scope, .. } => {
            output(
                scope.unwrap_or(0),
                name.split('.').next().expect("binding name"),
            );
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            if let Some(path) = path {
                if let Some(value) = fields.get(path.head) {
                    roots(value, path.tail, output);
                }
            }
        }
        ValueExpression::Member(base, field) => {
            roots(
                base,
                Some(&MemberPath {
                    head: field,
                    tail: path,
                }),
                output,
            );
        }
        ValueExpression::Default(value, fallback) => {
            roots(value, path, output);
            roots(fallback, path, output);
        }
        ValueExpression::Awaited(value) => roots(value, path, output),
        _ => {}
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "assigned_acquisition_roots_tests.rs"]
pub(super) mod tests;
