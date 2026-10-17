use super::*;
use std::sync::Arc;

#[cfg(feature = "test-instrumentation")]
thread_local! {
    pub(in super::super) static COLD: std::cell::Cell<[usize; 5]> = const { std::cell::Cell::new([0; 5]) };
}

#[derive(Default)]
pub(super) struct Names {
    ids: FxHashMap<Arc<str>, usize>,
    values: Vec<Arc<str>>,
}
impl Names {
    fn intern(&mut self, name: &str) -> usize {
        if let Some(id) = self.ids.get(name) {
            return *id;
        }
        let id = self.values.len();
        let name: Arc<str> = Arc::from(name);
        self.values.push(Arc::clone(&name));
        self.ids.insert(name, id);
        #[cfg(feature = "test-instrumentation")]
        COLD.with(|cold| {
            let previous = cold.get();
            cold.set([
                previous[0],
                self.values.len(),
                self.ids.capacity(),
                self.values.capacity(),
                self.values.iter().map(|name| name.len()).sum(),
            ]);
        });
        id
    }
    fn get(&self, id: usize) -> &str {
        &self.values[id]
    }
}

pub(in super::super) struct SingleNamedField {
    field: usize,
    name: usize,
    scope: Option<usize>,
    read: nested::RootRead,
}
impl From<SingleNamedField> for Origin {
    fn from(value: SingleNamedField) -> Self {
        Self::SingleNamedField(value)
    }
}
impl From<ValueExpression> for Origin {
    fn from(value: ValueExpression) -> Self {
        Self::Value(value)
    }
}
impl ReceiverBindingOrigins {
    pub(in super::super) fn single_named_field(
        &mut self,
        field: &str,
        name: &str,
        scope: Option<usize>,
        read: nested::RootRead,
    ) -> SingleNamedField {
        let names = self.names.get_or_insert_with(|| {
            #[cfg(feature = "test-instrumentation")]
            COLD.with(|cold| {
                let mut counts = cold.get();
                counts[0] += 1;
                cold.set(counts);
            });
            Box::default()
        });
        SingleNamedField {
            field: names.intern(field),
            name: names.intern(name),
            scope,
            read,
        }
    }
    pub(super) fn single_has_field(&self, value: &SingleNamedField, member: &str) -> bool {
        self.names
            .as_ref()
            .expect("single field owns names")
            .get(value.field)
            == member
    }
    pub(super) fn append_single<R: nested::RootCoordinate>(
        &self,
        value: &SingleNamedField,
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        output: &mut impl Extend<nested::ReceiverWrite<R>>,
    ) {
        if let Some((head, tail)) = path.split_first() {
            if head
                .as_ref()
                .is_none_or(|head| self.single_has_field(value, head))
            {
                let names = self.names.as_ref().expect("single field owns names");
                nested::write_named_receiver_references(
                    names.get(value.name),
                    value.scope,
                    value.read,
                    tail,
                    reads,
                    output,
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "mutable_origin_single_tests.rs"]
mod tests;
