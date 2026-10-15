use super::*;

type Receiver = ((usize, String), ValueMutationPath);
pub(super) type ReceiverRead = Option<(u32, Option<CallableId>)>;
pub(super) type RootRead = (u32, Option<CallableId>);
pub(super) type ReceiverWrite<R = RootRead> = (Receiver, Vec<ReceiverRead>, R);
/// Root tracking is selected once per physical writer. Ordinary receiver-read
/// timing remains present in both specializations.
pub(super) trait RootCoordinate: Copy {
    fn capture(read: RootRead) -> Self;
    fn read(self) -> Option<RootRead>;
}
impl RootCoordinate for RootRead {
    fn capture(read: RootRead) -> Self {
        read
    }
    fn read(self) -> Option<RootRead> {
        Some(self)
    }
}
impl RootCoordinate for () {
    fn capture(_: RootRead) -> Self {}
    fn read(self) -> Option<RootRead> {
        None
    }
}

pub(super) fn receiver_references(
    value: &ValueExpression,
    path: &[Option<String>],
    output: &mut Vec<Receiver>,
) {
    walk_references(value, path, 0, &mut |receiver, _, _| output.push(receiver));
}

pub(super) fn write_receiver_references<R: RootCoordinate>(
    value: &ValueExpression,
    path: &[Option<String>],
    reads: &[ReceiverRead],
    output: &mut impl Extend<ReceiverWrite<R>>,
) {
    walk_references(value, path, 0, &mut |receiver, acquired, read| {
        let retained = receiver.1.len() - acquired;
        let reads = std::iter::repeat_n(Some(read), acquired)
            .chain(reads[reads.len() - retained..].iter().copied())
            .collect();
        output.extend(std::iter::once((receiver, reads, R::capture(read))));
    });
}

fn walk_references(
    value: &ValueExpression,
    path: &[Option<String>],
    acquired: usize,
    output: &mut impl FnMut(Receiver, usize, (u32, Option<CallableId>)),
) {
    match value {
        ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } => {
            let mut names = name.split('.');
            let base = names.next().expect("binding name");
            let tail = names
                .map(|member| Some(member.to_string()))
                .chain(path.iter().cloned())
                .collect::<Vec<_>>();
            let acquired = acquired + tail.len() - path.len();
            output(
                ((scope.unwrap_or(0), base.to_string()), tail),
                acquired,
                (*offset, *context),
            );
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            if let Some((head, tail)) = path.split_first() {
                if let Some(head) = head {
                    if let Some(value) = fields.get(head) {
                        walk_references(value, tail, acquired.saturating_sub(1), output);
                    }
                } else {
                    for value in fields.values() {
                        walk_references(value, tail, acquired.saturating_sub(1), output);
                    }
                }
            }
        }
        ValueExpression::Member(base, field) => {
            let fields = std::iter::once(Some(field.clone()))
                .chain(path.iter().cloned())
                .collect::<Vec<_>>();
            walk_references(base, &fields, acquired + 1, output);
        }
        ValueExpression::Default(value, fallback) => {
            walk_references(value, path, acquired, output);
            walk_references(fallback, path, acquired, output);
        }
        ValueExpression::Awaited(value) => walk_references(value, path, acquired, output),
        _ => {}
    }
}
