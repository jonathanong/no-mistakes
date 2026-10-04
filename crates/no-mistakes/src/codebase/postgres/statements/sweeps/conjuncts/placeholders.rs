use super::super::super::value::is_placeholder_ident;
use sqlparser::ast::Ident;

pub(in crate::codebase::postgres::statements::sweeps) fn is_recovered_placeholder(
    ident: &Ident,
    positions: &[(u32, u32)],
) -> bool {
    ident.quote_style.is_none()
        && is_placeholder_ident(&ident.value)
        && positions.contains(&(ident.span.start.line as u32, ident.span.start.column as u32))
}
