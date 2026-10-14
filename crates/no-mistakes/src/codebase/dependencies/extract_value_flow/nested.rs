use super::*;

type Receiver = ((usize, String), ValueMutationPath);
pub(super) fn receiver_references(
    value: &ValueExpression,
    path: &[Option<String>],
    output: &mut Vec<Receiver>,
) {
    match value {
        ValueExpression::Named { name, scope, .. } => {
            let mut names = name.split('.');
            let base = names.next().expect("binding name");
            let tail = names
                .map(|member| Some(member.to_string()))
                .chain(path.iter().cloned())
                .collect();
            output.push(((scope.unwrap_or(0), base.to_string()), tail));
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            if let Some((head, tail)) = path.split_first() {
                if let Some(head) = head {
                    if let Some(value) = fields.get(head) {
                        receiver_references(value, tail, output);
                    }
                } else {
                    for value in fields.values() {
                        receiver_references(value, tail, output);
                    }
                }
            }
        }
        ValueExpression::Member(base, field) => {
            let fields = std::iter::once(Some(field.clone()))
                .chain(path.iter().cloned())
                .collect::<Vec<_>>();
            receiver_references(base, &fields, output);
        }
        ValueExpression::Default(value, fallback) => {
            receiver_references(value, path, output);
            receiver_references(fallback, path, output);
        }
        ValueExpression::Awaited(value) => receiver_references(value, path, output),
        _ => {}
    }
}
