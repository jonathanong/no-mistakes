//! Array argument accesses retain owner and bound dependencies without claiming scalar leaves.
use super::super::argument_base;
use super::Resolver;
use sqlparser::ast::{AccessExpr, Expr};

pub(super) fn collect(
    root: &Expr,
    access_chain: &[AccessExpr],
    resolver: &Resolver,
    indexed: &mut Vec<(usize, String)>,
    types: &mut Vec<String>,
    caller_only: bool,
    positions: super::super::super::super::super::value::PlaceholderPositions<'_>,
) -> Option<()> {
    let (base, bounds) = argument_base(root, access_chain)?;
    super::collect(
        &base,
        resolver,
        None,
        indexed,
        types,
        caller_only,
        positions,
    )?;
    for bound in bounds {
        super::collect(
            bound,
            resolver,
            None,
            indexed,
            types,
            caller_only,
            positions,
        )?;
    }
    Some(())
}
