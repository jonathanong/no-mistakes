use super::{
    EnqueueFactory, QueueFactoryRoles, QueueNameFactory, QueueWaitPatterns, WorkerFactory,
};
use serde::{Deserialize, Serialize};

pub fn deserialize_factories<'de, D>(deserializer: D) -> Result<QueueFactoryRoles, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut roles = FactoriesInput::deserialize(deserializer)?
        .into_roles()
        .map_err(serde::de::Error::custom)?;
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
        && roles.enqueue_and_wait.is_empty()
        && roles.job_scheduler.is_empty()
        && roles.enqueue_and_wait_patterns.is_empty()
        && !roles.forbid_enqueue_and_wait_declarations
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
        enqueue: roles.enqueue.clone(),
        worker: roles
            .worker
            .iter()
            .map(|factory| WorkerInput {
                callee: factory.callee.clone(),
                name: factory.name,
                processor: factory.processor,
            })
            .collect(),
        job_scheduler: roles
            .job_scheduler
            .iter()
            .map(super::scheduler::JobSchedulerInput::from)
            .collect(),
        enqueue_and_wait: roles.enqueue_and_wait.clone(),
        enqueue_and_wait_patterns: roles.enqueue_and_wait_patterns.patterns().to_vec(),
        forbid_enqueue_and_wait_declarations: roles.forbid_enqueue_and_wait_declarations,
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
    enqueue: Vec<EnqueueFactory>,
    worker: Vec<WorkerInput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    job_scheduler: Vec<super::scheduler::JobSchedulerInput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    enqueue_and_wait: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    enqueue_and_wait_patterns: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    forbid_enqueue_and_wait_declarations: bool,
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
struct WorkerInput {
    callee: String,
    name: usize,
    processor: usize,
}

impl FactoriesInput {
    fn into_roles(self) -> Result<QueueFactoryRoles, String> {
        Ok(match self {
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
                enqueue: super::enqueue::validate(body.enqueue)?,
                worker: body
                    .worker
                    .into_iter()
                    .map(|spec| WorkerFactory {
                        callee: spec.callee,
                        name: spec.name,
                        processor: spec.processor,
                    })
                    .collect(),
                job_scheduler: body
                    .job_scheduler
                    .into_iter()
                    .map(super::JobSchedulerFactory::from)
                    .collect(),
                enqueue_and_wait: body.enqueue_and_wait,
                enqueue_and_wait_patterns: QueueWaitPatterns::new(body.enqueue_and_wait_patterns)?,
                forbid_enqueue_and_wait_declarations: body.forbid_enqueue_and_wait_declarations,
                ..QueueFactoryRoles::default()
            },
        })
    }
}
