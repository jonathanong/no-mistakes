use super::super::super::frames;
use super::*;
impl Initials {
    pub fn remap(&mut self, indices: &FxHashMap<Environment, Environment>) {
        let env = |old: Environment| indices.get(&old).copied().unwrap_or(old);
        self.frames = std::mem::take(&mut self.frames)
            .into_iter()
            .map(|(id, mut values)| {
                for value in values.values_mut() {
                    frames::remap(value, indices);
                }
                (env(id), values)
            })
            .collect();
        for slots in self.objects.values_mut() {
            for value in slots {
                frames::remap(value, indices);
            }
        }
        for slots in self.extras.values_mut() {
            for value in slots.values_mut() {
                frames::remap(value, indices);
            }
        }
        self.mapped = std::mem::take(&mut self.mapped)
            .into_iter()
            .map(|(id, value)| (env(id), value))
            .collect();
        self.fresh = std::mem::take(&mut self.fresh)
            .into_iter()
            .map(|(id, value)| (env(id), value))
            .collect();
        self.captured = std::mem::take(&mut self.captured)
            .into_iter()
            .map(|(id, mut values)| {
                for origin in values.values_mut() {
                    *origin = env(*origin);
                }
                (env(id), values)
            })
            .collect();
    }
}
