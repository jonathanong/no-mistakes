use super::*;

pub(in super::super) trait ReceiverOutput<R>: Extend<ReceiverWrite<R>> {
    fn visit(&mut self) -> bool {
        true
    }
    fn active(&self) -> bool {
        true
    }
}
impl<R> ReceiverOutput<R> for Vec<ReceiverWrite<R>> {}
impl<R> ReceiverOutput<R> for super::super::reference_output::ReferenceOutput<ReceiverWrite<R>> {}
