/// Queue, enqueue, and worker factories declared in `queues.factories`.
///
/// A bare list of names stays the queue-factory form: each name is a call whose
/// first argument is the queue name. The object form names the argument index
/// for queue, enqueue, and worker wrappers separately.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueueFactoryRoles {
    pub queue: Vec<QueueNameFactory>,
    pub enqueue: Vec<EnqueueFactory>,
    pub worker: Vec<WorkerFactory>,
    /// Each project's `queues.waitScopes`, prefixed like other project globs.
    /// An empty scope list means every file of that project.
    pub wait_scopes: Vec<ProjectWaitScopes>,
}

#[path = "queues_wait.rs"]
mod wait;

pub use wait::ProjectWaitScopes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueNameFactory {
    pub callee: String,
    pub name: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnqueueFactory {
    pub callee: String,
    pub queue: usize,
    pub job: usize,
    pub options: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerFactory {
    pub callee: String,
    pub name: usize,
    pub processor: usize,
}

impl QueueFactoryRoles {
    pub fn queue_names(&self) -> Vec<String> {
        let mut names = self
            .queue
            .iter()
            .map(|factory| factory.callee.clone())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    /// Copy each project's wait scopes onto the roles the extractor receives.
    pub fn with_project_wait_scopes(mut self, config: &super::NoMistakesConfig) -> Self {
        wait::attach(&mut self.wait_scopes, config);
        self
    }

    /// Keep the first roles when `other` disagrees on an argument index.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        if let Some(message) = list_conflict(&self.queue, &other.queue, queue_same, "queue")
            .or_else(|| list_conflict(&self.enqueue, &other.enqueue, enqueue_same, "enqueue"))
            .or_else(|| list_conflict(&self.worker, &other.worker, worker_same, "worker"))
        {
            return Err(message);
        }
        self.queue.extend(other.queue.clone());
        self.enqueue.extend(other.enqueue.clone());
        self.worker.extend(other.worker.clone());
        wait::merge(&mut self.wait_scopes, &other.wait_scopes);
        self.dedup_callees();
        Ok(())
    }

    pub fn conflicts(&self) -> Option<String> {
        within(&self.queue, queue_same, "queue")
            .or_else(|| within(&self.enqueue, enqueue_same, "enqueue"))
            .or_else(|| within(&self.worker, worker_same, "worker"))
    }

    pub(super) fn dedup_callees(&mut self) {
        self.queue
            .sort_by(|left, right| left.callee.cmp(&right.callee));
        self.queue
            .dedup_by(|left, right| left.callee == right.callee);
        self.enqueue
            .sort_by(|left, right| left.callee.cmp(&right.callee));
        self.enqueue
            .dedup_by(|left, right| left.callee == right.callee);
        self.worker
            .sort_by(|left, right| left.callee.cmp(&right.callee));
        self.worker
            .dedup_by(|left, right| left.callee == right.callee);
        wait::dedup(&mut self.wait_scopes);
    }
}

fn queue_same(left: &QueueNameFactory, right: &QueueNameFactory) -> bool {
    left.name == right.name
}

fn enqueue_same(left: &EnqueueFactory, right: &EnqueueFactory) -> bool {
    left.queue == right.queue && left.job == right.job && left.options == right.options
}

fn worker_same(left: &WorkerFactory, right: &WorkerFactory) -> bool {
    left.name == right.name && left.processor == right.processor
}

fn list_conflict<T>(
    left: &[T],
    right: &[T],
    same: impl Fn(&T, &T) -> bool,
    kind: &str,
) -> Option<String>
where
    T: Callee,
{
    for item in right {
        if let Some(existing) = left
            .iter()
            .find(|candidate| candidate.callee() == item.callee())
        {
            if !same(existing, item) {
                return Some(conflict_message(item.callee(), kind));
            }
        }
    }
    None
}

fn within<T>(items: &[T], same: impl Fn(&T, &T) -> bool, kind: &str) -> Option<String>
where
    T: Callee,
{
    for (index, item) in items.iter().enumerate() {
        if let Some(message) =
            list_conflict(&items[..index], std::slice::from_ref(item), &same, kind)
        {
            return Some(message);
        }
    }
    None
}

fn conflict_message(callee: &str, kind: &str) -> String {
    format!("queue factory `{callee}` declares conflicting {kind} argument indexes")
}

trait Callee {
    fn callee(&self) -> &str;
}

impl Callee for QueueNameFactory {
    fn callee(&self) -> &str {
        &self.callee
    }
}

impl Callee for EnqueueFactory {
    fn callee(&self) -> &str {
        &self.callee
    }
}

impl Callee for WorkerFactory {
    fn callee(&self) -> &str {
        &self.callee
    }
}

#[path = "queues_parse.rs"]
mod parse;

pub use parse::{deserialize_factories, serialize_factories};
