use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::codebase::structured_value::expand_yaml_mapping_if_merged;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(super) struct RawWorkflows {
    docs: BTreeMap<String, Value>,
}

pub(super) enum RawBindingMap<'a> {
    Absent,
    Inherit,
    Mapping(Vec<(String, &'a Value)>),
    NonMapping(String),
}

impl RawWorkflows {
    pub(super) fn from_parsed(set: &ParsedWorkflowSet) -> Self {
        let mut docs = BTreeMap::new();
        for doc in &set.documents {
            let Ok(value) = &doc.value else {
                continue;
            };
            if let Ok(expanded) = expand_yaml_mapping_if_merged(value, doc.source.as_deref()) {
                docs.insert(doc.path.clone(), expanded.into_owned());
            }
        }
        Self { docs }
    }

    pub(super) fn job_field(
        &self,
        workflow_path: &str,
        job_key: &str,
        field: &str,
    ) -> RawBindingMap<'_> {
        let Some(raw) = self
            .job_mapping(workflow_path, job_key)
            .and_then(|job| job.get(Value::String(field.to_string())))
        else {
            return RawBindingMap::Absent;
        };
        classify_binding(raw, field == "secrets")
    }

    pub(super) fn has_job_field(&self, workflow_path: &str, job_key: &str, field: &str) -> bool {
        self.job_mapping(workflow_path, job_key)
            .is_some_and(|job| job.contains_key(Value::String(field.to_string())))
    }

    pub(super) fn callee_contract_field_non_mapping(
        &self,
        workflow_path: &str,
        field: &str,
    ) -> Option<String> {
        let call = self
            .docs
            .get(workflow_path)?
            .get("on")?
            .as_mapping()?
            .get("workflow_call")?
            .as_mapping()?;
        let raw = call.get(field)?;
        (!raw.is_null() && !raw.is_mapping()).then(|| describe_value(raw))
    }

    fn job_mapping(&self, workflow_path: &str, job_key: &str) -> Option<&serde_yaml::Mapping> {
        self.docs
            .get(workflow_path)?
            .get("jobs")?
            .as_mapping()?
            .get(Value::String(job_key.to_string()))?
            .as_mapping()
    }
}

pub(super) fn is_scalar(value: &Value) -> bool {
    matches!(value, Value::String(_) | Value::Number(_) | Value::Bool(_))
}

pub(super) fn describe_value(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| "a non-mapping value".to_string(), str::to_string)
}

fn classify_binding(raw: &Value, allow_inherit: bool) -> RawBindingMap<'_> {
    match raw {
        Value::Null => RawBindingMap::Absent,
        Value::String(text) if allow_inherit && text == "inherit" => RawBindingMap::Inherit,
        Value::Mapping(map) => {
            let entries = map
                .iter()
                .filter_map(|(key, value)| Some((key.as_str()?.to_string(), value)))
                .collect();
            RawBindingMap::Mapping(entries)
        }
        other => RawBindingMap::NonMapping(describe_value(other)),
    }
}
