use super::*;
use candidates::Candidates;

type Fields = std::sync::Arc<FxHashMap<String, ValueExpression>>;
#[derive(Default)]
struct Reader {
    next: usize,
    pending: Vec<usize>,
    has_alternatives: bool,
}
#[derive(Default)]
pub(super) struct Group {
    pub(super) events: std::collections::BTreeMap<u32, (Fields, bool)>,
    pub(super) pushes: Vec<usize>,
    push_cursor: usize,
    pushed: usize,
    coordinates: std::collections::BTreeSet<i128>,
    latest: Option<(Fields, usize)>,
    candidates: Candidates,
    readers: FxHashMap<Recipient, Reader>,
}
impl Group {
    pub(super) fn query(
        &mut self,
        calls: &[ValueCall],
        before: u32,
        recipient: Recipient,
    ) -> Option<Projection> {
        // The event map is drained as a cursor: no spread rebuilds its prefix.
        loop {
            let event = self.events.first_key_value().map(|(at, _)| *at);
            let push = self
                .pushes
                .get(self.push_cursor)
                .map(|index| calls[*index].offset);
            if push.is_some_and(|at| at < before && event.is_none_or(|event| at <= event)) {
                self.push(&calls[self.pushes[self.push_cursor]]);
                self.push_cursor += 1;
            } else if event.is_some_and(|at| at <= before) {
                let (_, (fields, guarded)) = self.events.pop_first().unwrap();
                self.event(fields, guarded)?;
            } else {
                break;
            }
        }
        let (fields, push_cursor) = self.latest.as_ref()?;
        let mut baseline = fields_entries(fields)?;
        let mut next = width(&baseline);
        for index in &self.pushes[*push_cursor..self.push_cursor] {
            #[cfg(feature = "test-instrumentation")]
            record(1, 1);
            for argument in calls[*index].arguments.iter() {
                flatten(argument, next, &mut baseline);
                next += 1;
            }
        }
        let baseline_length = next;
        let mut known = Candidates::default();
        for item in &baseline {
            known.insert(item.clone());
        }
        #[cfg(feature = "test-instrumentation")]
        record(4, known.items.len());
        let reader = self.readers.entry(recipient).or_default();
        let mut inspect = std::mem::take(&mut reader.pending);
        inspect.extend(reader.next..self.candidates.items.len());
        reader.next = self.candidates.items.len();
        let mut additional = Vec::new();
        for index in inspect {
            let item = &self.candidates.items[index];
            if known.contains(item) {
                // A current baseline candidate may become an older alternative later.
                reader.pending.push(index);
            } else {
                additional.push(item.clone());
            }
        }
        reader.has_alternatives |= !additional.is_empty();
        let has_alternatives = reader.has_alternatives;
        let minimum = (*self.coordinates.first()? + self.pushed as i128) as usize;
        #[cfg(feature = "test-instrumentation")]
        {
            record(3, additional.len());
            metrics::capture(&additional);
        }
        Some(Projection {
            baseline,
            additional,
            minimum,
            baseline_length,
            has_alternatives,
        })
    }
    fn event(&mut self, fields: Fields, guarded: bool) -> Option<()> {
        #[cfg(feature = "test-instrumentation")]
        record(0, 1);
        if !guarded {
            self.coordinates.clear();
            self.candidates = Candidates::default();
            self.readers.clear();
        }
        let entries = fields_entries(&fields)?;
        self.coordinates
            .insert(width(&entries) as i128 - self.pushed as i128);
        for entry in entries {
            self.candidates.insert(entry);
        }
        self.latest = Some((fields, self.push_cursor));
        Some(())
    }
    fn push(&mut self, call: &ValueCall) {
        #[cfg(feature = "test-instrumentation")]
        record(1, 1);
        for argument in call.arguments.iter() {
            let mut leaves = Vec::new();
            flatten(argument, 0, &mut leaves);
            for coordinate in &self.coordinates {
                #[cfg(feature = "test-instrumentation")]
                record(2, 1);
                let slot = (*coordinate + self.pushed as i128) as usize;
                for (_, value) in &leaves {
                    self.candidates.insert((slot, value.clone()));
                }
            }
            self.pushed += 1;
        }
    }
}
fn fields_entries(
    fields: &FxHashMap<String, ValueExpression>,
) -> Option<Vec<(usize, ValueExpression)>> {
    let mut entries = Vec::new();
    for (key, value) in fields {
        flatten(value, key.parse().ok()?, &mut entries);
    }
    Some(entries)
}
pub(super) fn width(entries: &[(usize, ValueExpression)]) -> usize {
    entries.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0)
}
