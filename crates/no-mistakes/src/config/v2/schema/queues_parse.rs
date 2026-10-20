use super::{EnqueueFactory, QueueFactoryRoles, QueueNameFactory, WorkerFactory};
use serde::{Deserialize, Serialize};

pub fn deserialize_factories<'de, D>(deserializer: D) -> Result<QueueFactoryRoles, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut roles = FactoriesInput::deserialize(deserializer)?.into_roles();
    if let Some(message) = roles.conflicts() {
        return Err(serde::de::Error::custom(message));
    }
    roles.dedup_callees();
    Ok(roles)
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
                options: factory.options,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    options: Option<usize>,
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
                        options: spec.options,
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
                ..QueueFactoryRoles::default()
            },
        }
    }
}
