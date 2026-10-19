use serde::{Deserialize, Serialize};

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
}

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

    pub fn merge(&mut self, other: &Self) {
        self.queue.extend(other.queue.clone());
        self.enqueue.extend(other.enqueue.clone());
        self.worker.extend(other.worker.clone());
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
    }
}

pub fn deserialize_factories<'de, D>(deserializer: D) -> Result<QueueFactoryRoles, D::Error>
where
    D: serde::Deserializer<'de>,
{
    FactoriesInput::deserialize(deserializer).map(FactoriesInput::into_roles)
}

pub fn serialize_factories<S>(roles: &QueueFactoryRoles, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if roles.enqueue.is_empty()
        && roles.worker.is_empty()
        && roles.queue.iter().all(|factory| factory.name == 0)
    {
        let names = roles
            .queue
            .iter()
            .map(|factory| factory.callee.as_str())
            .collect::<Vec<_>>();
        return names.serialize(serializer);
    }
    RolesBody {
        queue: roles
            .queue
            .iter()
            .map(|factory| {
                QueueEntry::Spec(QueueNameInput {
                    callee: factory.callee.clone(),
                    name: factory.name,
                })
            })
            .collect(),
        enqueue: roles
            .enqueue
            .iter()
            .map(|factory| EnqueueInput {
                callee: factory.callee.clone(),
                queue: factory.queue,
                job: factory.job,
            })
            .collect(),
        worker: roles
            .worker
            .iter()
            .map(|factory| WorkerInput {
                callee: factory.callee.clone(),
                name: factory.name,
                processor: factory.processor,
            })
            .collect(),
    }
    .serialize(serializer)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FactoriesInput {
    Names(Vec<String>),
    Roles(RolesBody),
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
struct RolesBody {
    queue: Vec<QueueEntry>,
    enqueue: Vec<EnqueueInput>,
    worker: Vec<WorkerInput>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum QueueEntry {
    Name(String),
    Spec(QueueNameInput),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QueueNameInput {
    callee: String,
    #[serde(default)]
    name: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EnqueueInput {
    callee: String,
    queue: usize,
    job: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerInput {
    callee: String,
    name: usize,
    processor: usize,
}

impl FactoriesInput {
    fn into_roles(self) -> QueueFactoryRoles {
        match self {
            Self::Names(names) => QueueFactoryRoles {
                queue: names
                    .into_iter()
                    .map(|callee| QueueNameFactory { callee, name: 0 })
                    .collect(),
                ..QueueFactoryRoles::default()
            },
            Self::Roles(body) => QueueFactoryRoles {
                queue: body
                    .queue
                    .into_iter()
                    .map(|entry| match entry {
                        QueueEntry::Name(callee) => QueueNameFactory { callee, name: 0 },
                        QueueEntry::Spec(spec) => QueueNameFactory {
                            callee: spec.callee,
                            name: spec.name,
                        },
                    })
                    .collect(),
                enqueue: body
                    .enqueue
                    .into_iter()
                    .map(|spec| EnqueueFactory {
                        callee: spec.callee,
                        queue: spec.queue,
                        job: spec.job,
                    })
                    .collect(),
                worker: body
                    .worker
                    .into_iter()
                    .map(|spec| WorkerFactory {
                        callee: spec.callee,
                        name: spec.name,
                        processor: spec.processor,
                    })
                    .collect(),
            },
        }
    }
}
