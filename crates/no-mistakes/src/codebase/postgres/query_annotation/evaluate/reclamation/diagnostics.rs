use super::*;
impl Reclamation {
    pub(super) fn record_reference_insert(&mut self, references: &References) {
        self.pending_reference_work[0] += 1;
        self.pending_reference_work[1] += references.arguments.len() as u64;
        self.pending_reference_work[2] += references.builders.len() as u64;
    }
    pub(in super::super) fn report_reference_metadata(&mut self) {
        let work = std::mem::take(&mut self.pending_reference_work);
        let Some(observer) = self.observer.get_or_init(crate::diagnostics::current) else {
            return;
        };
        for (label, amount) in [
            "postgres.annotation.reference_locations_built",
            "postgres.annotation.argument_reference_edges_built",
            "postgres.annotation.builder_holder_edges_built",
        ]
        .into_iter()
        .zip(work)
        {
            observer.increment(label, amount);
        }
        // These sum each evaluator's observed peaks, not process-wide RSS or bytes.
        let values = [
            self.locations.len(),
            self.locations.capacity(),
            self.incoming.len(),
            self.pins.len(),
            self.callback_keys.len(),
        ];
        let labels = [
            "postgres.annotation.reference_locations_peak_sum",
            "postgres.annotation.reference_capacity_peak_sum",
            "postgres.annotation.reference_arguments_peak_sum",
            "postgres.annotation.reference_pins_peak_sum",
            "postgres.annotation.callback_key_frames_peak_sum",
        ];
        for index in 0..values.len() {
            if values[index] > self.peaks[index] {
                observer.increment(labels[index], (values[index] - self.peaks[index]) as u64);
                self.peaks[index] = values[index];
            }
        }
    }
    pub(super) fn shrink_reference_storage(&mut self) {
        macro_rules! shrink {
            ($map:expr) => {
                if $map.capacity() > $map.len().saturating_mul(4).max(16) {
                    $map.shrink_to_fit();
                }
            };
        }
        shrink!(self.locations);
        shrink!(self.owners);
        shrink!(self.incoming);
        shrink!(self.roots);
        shrink!(self.holders);
        shrink!(self.masks);
        shrink!(self.callback_keys);
        shrink!(self.candidates);
        shrink!(self.builder_candidates);
    }
}
