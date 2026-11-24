use super::super::extract_model::{JobOptionEvidence, JobOptionSite};
use oxc_ast::ast::{
    Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKind,
};

pub(in crate::queue) const KIND: &str = "unbounded-job-retention";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::queue) enum RetentionValue {
    #[default]
    Absent,
    Bounded,
    Unbounded,
    Unknown,
}

impl RetentionValue {
    fn overlay(self, other: Self) -> Self {
        match other {
            Self::Absent => self,
            other => other,
        }
    }

    fn reconcile(self, other: Self) -> Self {
        if self == other {
            self
        } else {
            Self::Unknown
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::queue) struct RetentionEvidence {
    complete: RetentionValue,
    fail: RetentionValue,
}

impl RetentionEvidence {
    pub(in crate::queue) const UNKNOWN: Self = Self {
        complete: RetentionValue::Unknown,
        fail: RetentionValue::Unknown,
    };

    pub(in crate::queue) fn overlay(&mut self, other: Self) {
        self.complete = self.complete.overlay(other.complete);
        self.fail = self.fail.overlay(other.fail);
    }

    pub(super) fn reconcile(&mut self, other: Self) {
        self.complete = self.complete.reconcile(other.complete);
        self.fail = self.fail.reconcile(other.fail);
    }

    pub(super) fn get(self, key: &str) -> RetentionValue {
        match key {
            "removeOnComplete" => self.complete,
            "removeOnFail" => self.fail,
            _ => RetentionValue::Unknown,
        }
    }

    pub(super) fn record(&mut self, name: &str, property: &ObjectProperty<'_>) {
        let value = match name {
            "removeOnComplete" => &mut self.complete,
            "removeOnFail" => &mut self.fail,
            _ => return,
        };
        *value = match property.kind {
            PropertyKind::Init => classify(property.value.get_inner_expression()),
            _ => RetentionValue::Unknown,
        };
    }
}

fn classify(value: &Expression<'_>) -> RetentionValue {
    match value {
        Expression::BooleanLiteral(value) if value.value => RetentionValue::Bounded,
        Expression::BooleanLiteral(_) => RetentionValue::Unbounded,
        Expression::NumericLiteral(_) => RetentionValue::Bounded,
        Expression::ObjectExpression(object) => {
            let (age, count) = bounds(object);
            match (age, count) {
                (RetentionValue::Bounded, _) | (_, RetentionValue::Bounded) => {
                    RetentionValue::Bounded
                }
                (RetentionValue::Absent, RetentionValue::Absent) => RetentionValue::Unbounded,
                _ => RetentionValue::Unknown,
            }
        }
        _ => RetentionValue::Unknown,
    }
}

fn bounds(object: &ObjectExpression<'_>) -> (RetentionValue, RetentionValue) {
    let unknown = (RetentionValue::Unknown, RetentionValue::Unknown);
    let mut bounds = (RetentionValue::Absent, RetentionValue::Absent);
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                let value = if property.kind == PropertyKind::Init
                    && matches!(
                        property.value.get_inner_expression(),
                        Expression::NumericLiteral(_)
                    ) {
                    RetentionValue::Bounded
                } else {
                    RetentionValue::Unknown
                };
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("age") => bounds.0 = value,
                    Some("count") => bounds.1 = value,
                    Some(_) => {}
                    None => bounds = unknown,
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                let (age, count) = match spread.argument.get_inner_expression() {
                    Expression::ObjectExpression(inner) => self::bounds(inner),
                    _ => unknown,
                };
                bounds.0 = bounds.0.overlay(age);
                bounds.1 = bounds.1.overlay(count);
            }
        }
    }
    bounds
}

pub(super) fn message(keys: &[&str]) -> String {
    format!(
        "job retention is unbounded for {}. Set a count or age limit.",
        keys.join(", ")
    )
}

pub(super) fn unbounded_at_site(
    site: &JobOptionSite,
    inherited: Option<&JobOptionEvidence>,
    key: &str,
) -> bool {
    match site.evidence.retention.get(key) {
        RetentionValue::Unbounded => true,
        RetentionValue::Absent => {
            site.queue_name.is_some()
                && inherited
                    .is_none_or(|defaults| defaults.retention.get(key) == RetentionValue::Absent)
        }
        RetentionValue::Bounded | RetentionValue::Unknown => false,
    }
}
