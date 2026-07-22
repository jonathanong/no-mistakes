use super::{ResourcePath, ResourcePathBase};
use oxc_ast::ast::{Argument, Expression, NewExpression};

pub(super) fn static_string(arg: &Argument<'_>) -> Option<String> {
    match arg {
        Argument::StringLiteral(s) => Some(s.value.to_string()),
        Argument::TemplateLiteral(template) if template.expressions.is_empty() => Some(
            template
                .quasis
                .iter()
                .map(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .unwrap_or(&quasi.value.raw)
                        .as_ref()
                })
                .collect(),
        ),
        _ => None,
    }
}

pub(super) fn static_new_module_url(new: &NewExpression<'_>) -> Option<ResourcePath> {
    let Expression::Identifier(callee) = &new.callee else {
        return None;
    };
    if callee.name != "URL" || new.arguments.len() != 2 || !is_import_meta_url(&new.arguments[1]) {
        return None;
    }
    static_string(&new.arguments[0])
        .and_then(|value| decode_module_url_path(&value))
        .map(|value| ResourcePath {
            value,
            base: ResourcePathBase::SourceModule,
        })
}

fn is_import_meta_url(argument: &Argument<'_>) -> bool {
    matches!(argument, Argument::StaticMemberExpression(member)
        if member.property.name == "url"
            && matches!(&member.object, Expression::MetaProperty(meta)
                if meta.meta.name == "import" && meta.property.name == "meta"))
}

/// `new URL(relative, import.meta.url)` treats percent escapes as URL escapes,
/// unlike a plain filesystem string. Reject encoded path separators because
/// Node rejects them too, and reject absolute/network URL forms rather than
/// accidentally turning them into local paths.
pub(super) fn decode_module_url_path(value: &str) -> Option<String> {
    if value.starts_with("//")
        || value.contains("://")
        || value.starts_with("file:")
        || value.as_bytes().get(1) == Some(&b':')
    {
        return None;
    }
    let value = value.split(['?', '#']).next().unwrap_or_default();
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        if raw[index] != b'%' {
            bytes.push(raw[index]);
            index += 1;
            continue;
        }
        let high = u8::try_from(char::from(*raw.get(index + 1)?).to_digit(16)?).ok()?;
        let low = u8::try_from(char::from(*raw.get(index + 2)?).to_digit(16)?).ok()?;
        let decoded = (high << 4) | low;
        if matches!(decoded, b'/' | b'\\') {
            return None;
        }
        bytes.push(decoded);
        index += 3;
    }
    String::from_utf8(bytes).ok()
}
