use super::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

/// `is_shadowed` reports whether a name is currently bound to something
/// other than the global trusted SQL tag (e.g. a same-file helper's own
/// parameter named `sql`) — matching the tag purely by spelling, as
/// [`is_sql_tag`] alone does, would trust a caller-controlled tag function
/// passed in under that name.
///
/// A tagged template with interpolated values is untrusted unless the tag
/// is really the built-in `sql` concatenation tag: it parameterizes
/// interpolated values instead of splicing them into the text, which is
/// what makes trusting its quasi text safe even though the call has
/// arguments. Nothing else — `String.raw` included — offers that guarantee,
/// so any other tag with interpolations is untrusted.
///
/// A tagged template with zero interpolated values carries no runtime data
/// at all, so the only question left is whether the tag returns its quasi
/// text unchanged. The unshadowed `sql` tag and the built-in `String.raw`
/// both do so deterministically; any other tag — same-spelled-but-shadowed
/// `sql` included — is an arbitrary function that can ignore its template
/// argument and return something else entirely, so its quasi text can't be
/// trusted either.
pub(super) fn interpolating_untrusted_tag(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
) -> bool {
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(expr) else {
        return false;
    };
    if tagged.quasi.expressions.is_empty() {
        return !is_sql_tag(&tagged.tag, is_shadowed)
            && !is_string_raw_tag(&tagged.tag, is_shadowed);
    }
    !is_sql_tag(&tagged.tag, is_shadowed)
}

/// The built-in `String.raw` tag: a fixed, well-known JS semantic (return
/// the raw template text unchanged) rather than an arbitrary function, so
/// its zero-interpolation output can be trusted to be its quasi text —
/// unless `String` itself is shadowed (e.g. a same-file helper's own
/// parameter named `String`), in which case `.raw` is a property access on
/// whatever arbitrary value that binding holds, not the real built-in,
/// matching how [`is_sql_tag`] already treats a shadowed `sql`.
fn is_string_raw_tag(tag: &Expression<'_>, is_shadowed: &mut impl FnMut(&str) -> bool) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(tag) else {
        return false;
    };
    if member.property.name != "raw" {
        return false;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    ident.name == "String" && !is_shadowed(ident.name.as_str())
}

/// Syntactic-only counterpart of [`is_string_raw_tag`]: whether `tag` is
/// spelled `String.raw`, regardless of whether `String` is shadowed. Used
/// once a tagged template's trust has already been settled — by
/// [`interpolating_untrusted_tag`], which does consult shadowing — to pick
/// which quasi form (raw or cooked) reflects what `String.raw`'s own
/// well-known semantics would actually produce.
pub(super) fn is_string_raw_tag_spelling(tag: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(tag) else {
        return false;
    };
    member.property.name == "raw"
        && matches!(
            unwrap_ts_wrappers(&member.object),
            Expression::Identifier(ident) if ident.name == "String"
        )
}

pub(super) fn kind_for_const(sql: String, is_const: bool) -> (Option<String>, EmbeddedSqlKind) {
    if is_const {
        (Some(sql), EmbeddedSqlKind::ImmutableLocal)
    } else {
        (Some(sql), EmbeddedSqlKind::Dynamic)
    }
}

fn is_sql_tag(tag: &Expression<'_>, is_shadowed: &mut impl FnMut(&str) -> bool) -> bool {
    match unwrap_ts_wrappers(tag) {
        Expression::Identifier(ident) if ident.name.eq_ignore_ascii_case("sql") => {
            !is_shadowed(ident.name.as_str())
        }
        _ => false,
    }
}
