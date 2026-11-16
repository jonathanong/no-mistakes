use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum TextOwner<'a> {
    Local(&'a str, usize),
    Namespace(&'a str, &'a str),
}

fn owner_identity(value: &ValueExpression) -> Option<TextOwner<'_>> {
    match value {
        ValueExpression::Named { name, scope, .. } => {
            Some(TextOwner::Local(name, scope.unwrap_or(0)))
        }
        ValueExpression::Member(base, exported) => {
            let ValueExpression::Named { name, .. } = base.as_ref() else {
                return None;
            };
            Some(TextOwner::Namespace(name, exported))
        }
        _ => None,
    }
}

fn text_parts(value: &ValueExpression) -> &[ValueExpression] {
    match value {
        ValueExpression::Derived(parts) => parts,
        other => std::slice::from_ref(other),
    }
}

fn member_owner(value: &ValueExpression) -> Option<TextOwner<'_>> {
    match value {
        ValueExpression::Member(owner, key) if key == "text" => owner_identity(owner),
        _ => None,
    }
}

pub(in super::super) fn text_owner<'a>(
    bindings: &Bindings<'a>,
    call: &SqlCall<'_>,
) -> Option<(TextOwner<'a>, bool)> {
    let value_call = bindings.value_call(&call.file, call.span)?;
    let owner = text_parts(value_call.arguments.first()?)
        .iter()
        .find_map(member_owner)?;
    Some((owner, text_owner_for(bindings, call, owner)?))
}

pub(in super::super) fn text_owner_for(
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    wanted: TextOwner<'_>,
) -> Option<bool> {
    let value_call = bindings.value_call(&call.file, call.span)?;
    let paired = value_call.arguments.get(1).is_some_and(|arg| {
        matches!(arg, ValueExpression::Member(inner, key) if key == "values"
            && owner_identity(inner) == Some(wanted))
    });
    Some(paired)
}
