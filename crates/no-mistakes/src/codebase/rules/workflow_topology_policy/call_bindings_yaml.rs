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
            let expanded = expand_yaml_mapping_if_merged(value, doc.source.as_deref())
                .map(std::borrow::Cow::into_owned)
                .unwrap_or_else(|_| value.clone());
            docs.insert(doc.path.clone(), expanded);
        }
        Self { docs }
    }

    pub(super) fn job_field(&self, workflow_path: &str, job_key: &str, field: &str) -> RawBindingMap<'_> {
        let Some(job) = self
            .docs
            .get(workflow_path)
            .and_then(|doc| doc.get("jobs"))
            .and_then(Value::as_mapping)
            .and_then(|jobs| lookup_key(jobs, job_key))
            .and_then(Value::as_mapping)
        else {
            return RawBindingMap::Absent;
        };
        let Some(raw) = lookup_key(job, field) else {
            return RawBindingMap::Absent;
        };
        classify_binding(raw, field == "secrets")
    }

    pub(super) fn has_job_field(&self, workflow_path: &str, job_key: &str, field: &str) -> bool {
        self.docs
            .get(workflow_path)
            .and_then(|doc| doc.get("jobs"))
            .and_then(Value::as_mapping)
            .and_then(|jobs| lookup_key(jobs, job_key))
            .and_then(Value::as_mapping)
            .is_some_and(|job| lookup_key(job, field).is_some())
    }

    pub(super) fn callee_contract_field_non_mapping(
        &self,
        workflow_path: &str,
        field: &str,
    ) -> Option<String> {
        let doc = self.docs.get(workflow_path)?;
        let on = doc
            .get("on")
            .or_else(|| doc.as_mapping()?.get(Value::Bool(true)))?;
        let call = on.as_mapping()?.get("workflow_call")?.as_mapping()?;
        let raw = call.get(field)?;
        if raw.is_null() || raw.is_mapping() {
            None
        } else {
            Some(describe_value(raw))
        }
    }
}

pub(super) fn is_scalar(value: &Value) -> bool {
    matches!(value, Value::String(_) | Value::Number(_) | Value::Bool(_))
}

pub(super) fn describe_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Sequence(_) => "a sequence".to_string(),
        Value::Mapping(_) => "a mapping".to_string(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".to_string(),
        Value::Tagged(_) => "a tagged value".to_string(),
    }
}

fn classify_binding(raw: &Value, allow_inherit: bool) -> RawBindingMap<'_> {
    match raw {
        Value::Null => RawBindingMap::Absent,
        Value::String(text) if allow_inherit && text == "inherit" => RawBindingMap::Inherit,
        Value::Mapping(map) => {
            let entries = map
                .iter()
                .filter_map(|(key, value)| Some((key_name(key)?, value)))
                .collect();
            RawBindingMap::Mapping(entries)
        }
        other => RawBindingMap::NonMapping(describe_value(other)),
    }
}

fn lookup_key<'a>(map: &'a serde_yaml::Mapping, target: &str) -> Option<&'a Value> {
    map.get(Value::String(target.to_string())).or_else(|| {
        map.iter()
            .find_map(|(key, value)| (key_name(key).as_deref() == Some(target)).then_some(value))
    })
}

fn key_name(key: &Value) -> Option<String> {
    match key {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}
