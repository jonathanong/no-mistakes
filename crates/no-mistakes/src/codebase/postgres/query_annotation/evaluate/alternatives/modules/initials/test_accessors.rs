use super::*;
impl Initials {
    pub fn arguments(&self) -> impl Iterator<Item = u64> + '_ {
        self.objects.iter().copied()
    }
    pub fn roots(&self) -> impl Iterator<Item = Environment> + '_ {
        self.frames.values().copied()
    }
}
