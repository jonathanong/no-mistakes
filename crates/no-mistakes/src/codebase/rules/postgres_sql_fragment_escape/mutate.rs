use super::fragments::Fragment;
use super::names::expr_module;
use super::resolve::{member_of, use_expr, FileCtx};
use crate::codebase::dependencies::extract::{ValueCall, ValueExpression};

pub(super) fn receiver_mutations(ctx: &FileCtx<'_>, call: &ValueCall) -> Vec<Fragment> {
    let ValueExpression::Member(base, name) = &call.callee else {
        return Vec::new();
    };
    if name == "append" {
        return use_expr(ctx, base, 0);
    }
    nested_member_call(ctx, base)
}

fn nested_member_call(ctx: &FileCtx<'_>, expr: &ValueExpression) -> Vec<Fragment> {
    let ValueExpression::Member(inner, member) = expr else {
        return Vec::new();
    };
    if expr_module(ctx, inner, 0).is_some() {
        return member_of(ctx, inner, member, 0);
    }
    let peeled = nested_member_call(ctx, inner);
    if !peeled.is_empty() {
        peeled
    } else {
        use_expr(ctx, inner, 0)
    }
}
