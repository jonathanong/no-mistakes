mod parse;
pub(super) use parse::parse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Marker {
    pub(super) order: Vec<usize>,
    pub(super) command: Command,
}

#[cfg(test)]
#[path = "transactions/tests.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Command {
    Begin,
    Commit,
    CommitAndChain,
    Rollback,
    RollbackAndChain,
    Savepoint(String),
    RollbackTo(String),
    Release(String),
}

/// Keep only table events that are not inside a known rolled-back transaction.
/// Events without a reliable lexical position, and events in transactions that
/// remain open at EOF, are retained conservatively.
pub(super) fn retained_events(orders: &[&[usize]], markers: &[Marker]) -> Vec<bool> {
    let mut retained = vec![true; orders.len()];
    let mut transaction = Transaction::default();
    let mut marker_at = 0;
    for (index, order) in orders.iter().enumerate() {
        while markers
            .get(marker_at)
            .is_some_and(|marker| marker.order.as_slice() <= *order)
        {
            transaction.apply(&markers[marker_at].command, &mut retained);
            marker_at += 1;
        }
        if order.contains(&usize::MAX) {
            continue;
        }
        if transaction.active {
            transaction.pending.push(index);
        }
    }
    for marker in &markers[marker_at..] {
        transaction.apply(&marker.command, &mut retained);
    }
    transaction.commit_open();
    retained
}

#[derive(Default)]
struct Transaction {
    active: bool,
    pending: Vec<usize>,
    savepoints: Vec<(String, usize)>,
}

impl Transaction {
    fn apply(&mut self, command: &Command, retained: &mut [bool]) {
        match command {
            Command::Begin if !self.active => {
                self.active = true;
                self.pending.clear();
                self.savepoints.clear();
            }
            Command::Commit if self.active => self.commit(retained, false),
            Command::CommitAndChain if self.active => self.commit(retained, true),
            Command::Rollback if self.active => self.rollback(retained, false),
            Command::RollbackAndChain if self.active => self.rollback(retained, true),
            Command::Savepoint(name) if self.active => {
                self.savepoints.push((name.clone(), self.pending.len()));
            }
            Command::RollbackTo(name) if self.active => self.rollback_to(name, retained),
            Command::Release(name) if self.active => self.release(name),
            _ => {}
        }
    }

    fn commit(&mut self, retained: &mut [bool], chain: bool) {
        for index in self.pending.drain(..) {
            retained[index] = true;
        }
        self.savepoints.clear();
        self.active = chain;
    }

    fn rollback(&mut self, retained: &mut [bool], chain: bool) {
        for index in self.pending.drain(..) {
            retained[index] = false;
        }
        self.savepoints.clear();
        self.active = chain;
    }

    fn rollback_to(&mut self, name: &str, retained: &mut [bool]) {
        let Some(savepoint) = self
            .savepoints
            .iter()
            .rposition(|(current, _)| current == name)
        else {
            return;
        };
        let keep = self.savepoints[savepoint].1;
        for index in self.pending.drain(keep..) {
            retained[index] = false;
        }
        self.savepoints.truncate(savepoint + 1);
    }

    fn release(&mut self, name: &str) {
        if let Some(savepoint) = self
            .savepoints
            .iter()
            .rposition(|(current, _)| current == name)
        {
            self.savepoints.truncate(savepoint);
        }
    }

    fn commit_open(&mut self) {
        // The transaction may continue in another migration file. Do not discard
        // events merely because this per-file fact pass ends before its outcome.
        self.pending.clear();
    }
}
