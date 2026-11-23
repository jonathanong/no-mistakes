use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EnqueueSelector {
    Positional(usize),
    Property(EnqueuePropertySelector),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnqueuePropertySelector {
    pub arg: usize,
    pub property: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnqueueFactory {
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<EnqueueSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_name: Option<String>,
    pub job: EnqueueSelector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<EnqueueSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<EnqueueSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_on_returned_call: Option<usize>,
}

impl EnqueueFactory {
    pub(crate) fn positional(&self) -> Option<(usize, usize, Option<usize>)> {
        let (Some(EnqueueSelector::Positional(queue)), EnqueueSelector::Positional(job)) =
            (&self.queue, &self.job)
        else {
            return None;
        };
        let options = match self.options {
            Some(EnqueueSelector::Positional(index)) => Some(index),
            None => None,
            _ => return None,
        };
        (self.defaults.is_none() && self.options_on_returned_call.is_none())
            .then_some((*queue, *job, options))
    }
}

pub(super) fn validate(factories: Vec<EnqueueFactory>) -> Result<Vec<EnqueueFactory>, String> {
    for factory in &factories {
        if factory.queue.is_some() == factory.queue_name.is_some() {
            return Err(format!(
                "enqueue factory `{}` requires exactly one of queue and queueName",
                factory.callee
            ));
        }
    }
    Ok(factories)
}
