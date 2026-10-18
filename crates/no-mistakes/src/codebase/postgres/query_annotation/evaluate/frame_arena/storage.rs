use super::{Environment, Scope};

const SHIFT: u32 = usize::BITS / 2;
const MASK: usize = (1usize << SHIFT) - 1;

#[derive(Clone)]
struct Slot {
    generation: usize,
    birth: u64,
    scope: Option<Scope>,
}

#[derive(Clone, Default)]
pub(super) struct Storage {
    slots: Vec<Slot>,
    free: Vec<usize>,
}

impl Storage {
    pub(super) fn insert(&mut self, scope: Scope, birth: u64) -> Environment {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index];
            slot.scope = Some(scope);
            slot.birth = birth;
            (slot.generation << SHIFT) | index
        } else {
            let index = self.slots.len();
            assert!(index <= MASK, "frame slot identity exhausted");
            self.slots.push(Slot {
                generation: 0,
                birth,
                scope: Some(scope),
            });
            index
        }
    }

    fn slot(&self, env: Environment) -> Option<&Slot> {
        let slot = self.slots.get(env & MASK)?;
        (slot.generation == env >> SHIFT && slot.scope.is_some()).then_some(slot)
    }

    pub(super) fn birth(&self, env: Environment) -> Option<u64> {
        Some(self.slot(env)?.birth)
    }

    pub(super) fn get(&self, env: Environment) -> Option<&Scope> {
        self.slot(env)?.scope.as_ref()
    }

    pub(super) fn get_mut(&mut self, env: Environment) -> Option<&mut Scope> {
        self.slot(env)?;
        self.slots[env & MASK].scope.as_mut()
    }

    pub(super) fn remove(&mut self, env: Environment) -> Option<Scope> {
        self.slot(env)?;
        let index = env & MASK;
        let slot = &mut self.slots[index];
        let scope = slot.scope.take();
        // Exhausted tags retire a slot instead of aliasing a stale handle.
        if slot.generation < MASK {
            slot.generation += 1;
            self.free.push(index);
        }
        scope
    }

    pub(super) fn ids(&self) -> impl Iterator<Item = Environment> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.scope
                .as_ref()
                .map(|_| (slot.generation << SHIFT) | index)
        })
    }

    pub(super) fn len(&self) -> usize {
        self.slots.len()
    }
}
