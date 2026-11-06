use super::*;

#[path = "nested_output.rs"]
mod receiver_output;
pub(super) use receiver_output::ReceiverOutput;

#[cfg(test)]
#[path = "nested_output_tests.rs"]
mod output_tests;

type Receiver = ((usize, String), ValueMutationPath);
pub(super) type ReceiverRead = Option<(u32, Option<CallableId>)>;
pub(super) type RootRead = (u32, Option<CallableId>);
pub(super) type ReceiverWrite<R = RootRead> = (Receiver, Vec<ReceiverRead>, R);
pub(super) type ForeignMemberReferences = Vec<(ReceiverWrite, Option<CallableId>, (u32, u32))>;
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
    walk_references(value, path, 0, &mut |value| {
        if let Some((receiver, _, _)) = value {
            output.push(receiver);
        }
        true
    });
}

pub(super) fn write_receiver_references<R: RootCoordinate>(
    value: &ValueExpression,
    path: &[Option<String>],
    reads: &[ReceiverRead],
    output: &mut impl ReceiverOutput<R>,
) {
    walk_references(value, path, 0, &mut |value| match value {
        None => output.visit(),
        Some((receiver, acquired, read)) => {
            append_write(receiver, acquired, read, reads, output);
            output.active()
        }
    });
}

fn walk_references(
    value: &ValueExpression,
    path: &[Option<String>],
    acquired: usize,
    output: &mut impl FnMut(Option<(Receiver, usize, RootRead)>) -> bool,
) -> bool {
    if !output(None) {
        return false;
    }
    match value {
        ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } => {
            return named_reference(
                name,
                *scope,
                (*offset, *context),
                path,
                acquired,
                &mut |receiver, acquired, read| output(Some((receiver, acquired, read))),
            );
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            if let Some((head, tail)) = path.split_first() {
                if let Some(head) = head {
                    if let Some(value) = fields.get(head) {
                        if !walk_references(value, tail, acquired.saturating_sub(1), output) {
                            return false;
                        }
                    }
                } else {
                    for value in fields.values() {
                        if !walk_references(value, tail, acquired.saturating_sub(1), output) {
                            return false;
                        }
                    }
                }
            }
        }
        ValueExpression::Member(base, field) => {
            let fields = std::iter::once(Some(field.clone()))
                .chain(path.iter().cloned())
                .collect::<Vec<_>>();
            return walk_references(base, &fields, acquired + 1, output);
        }
        ValueExpression::Default(value, fallback) | ValueExpression::Either(value, fallback) => {
            return walk_references(value, path, acquired, output)
                && walk_references(fallback, path, acquired, output);
        }
        ValueExpression::Awaited(value) => return walk_references(value, path, acquired, output),
        _ => {}
    }
    true
}

/// Both full values and compact root facts share named receiver/read projection.
pub(super) fn write_named_receiver_references<R: RootCoordinate>(
    name: &str,
    scope: Option<usize>,
    read: RootRead,
    path: &[Option<String>],
    reads: &[ReceiverRead],
    output: &mut impl ReceiverOutput<R>,
) {
    if !output.visit() {
        return;
    }
    named_reference(
        name,
        scope,
        read,
        path,
        0,
        &mut |receiver, acquired, read| {
            append_write(receiver, acquired, read, reads, output);
            output.active()
        },
    );
}
fn named_reference(
    name: &str,
    scope: Option<usize>,
    read: RootRead,
    path: &[Option<String>],
    acquired: usize,
    output: &mut impl FnMut(Receiver, usize, RootRead) -> bool,
) -> bool {
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
        read,
    )
}
fn append_write<R: RootCoordinate>(
    receiver: Receiver,
    acquired: usize,
    read: RootRead,
    reads: &[ReceiverRead],
    output: &mut impl ReceiverOutput<R>,
) {
    let retained = receiver.1.len() - acquired;
    let reads = std::iter::repeat_n(Some(read), acquired)
        .chain(reads[reads.len() - retained..].iter().copied())
        .collect();
    output.extend(std::iter::once((receiver, reads, R::capture(read))));
}
