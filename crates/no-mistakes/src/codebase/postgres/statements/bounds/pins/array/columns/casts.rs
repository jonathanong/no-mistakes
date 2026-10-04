//! Cast result shape and trusted reducer argument traversal are distinct proofs.
use super::super::super::super::super::value::PlaceholderPositions;
use super::super::super::Resolver;
use sqlparser::ast::{DataType, Expr};

pub(super) fn collect(
    expr: &Expr,
    data_type: &DataType,
    mut resolver_out: Option<&mut Vec<(usize, String)>>,
    resolver: &Resolver,
    indexed: &mut Vec<(usize, String)>,
    types: &mut Vec<String>,
    caller_only: bool,
    positions: PlaceholderPositions<'_>,
) -> Option<()> {
    if resolver_out.is_none() {
        return super::collect(expr, resolver, None, indexed, types, caller_only, positions);
    }
    match data_type {
        DataType::Array(_) => {
            // Decoding scalar text does not establish finite array cardinality.
            if let Some(array) = super::super::constructor(expr) {
                for element in &array.elem {
                    super::collect(
                        element,
                        resolver,
                        resolver_out.as_deref_mut(),
                        indexed,
                        types,
                        caller_only,
                        positions,
                    )?;
                }
                return Some(());
            }
            super::collect(
                expr,
                resolver,
                resolver_out,
                indexed,
                types,
                true,
                positions,
            )
        }
        _ => {
            if matches!(data_type, DataType::Custom(_, _)) {
                types.push(data_type.to_string());
            }
            if caller_only {
                super::collect(
                    expr,
                    resolver,
                    resolver_out,
                    indexed,
                    types,
                    true,
                    positions,
                )?;
            }
            Some(())
        }
    }
}
