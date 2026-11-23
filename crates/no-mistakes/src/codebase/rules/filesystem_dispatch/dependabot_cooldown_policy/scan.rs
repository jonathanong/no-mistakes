use super::{Options, RuleFinding, RULE_ID};
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use globset::GlobSet;
use serde_yaml::Value;

const DAYS: [&str; 4] = [
    "default-days",
    "semver-major-days",
    "semver-minor-days",
    "semver-patch-days",
];
const PR_LIMIT: &str = "open-pull-requests-limit";

pub(super) fn check(
    file: &str,
    updates: Option<&[Value]>,
    locations: Option<&ConfigLocations>,
    options: &Options,
    minimum_days: u64,
    allowed: Option<&GlobSet>,
    message: Option<&str>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (index, entry) in updates.into_iter().flatten().enumerate() {
        let ecosystem = entry.get("package-ecosystem").and_then(Value::as_str);
        if options
            .exempt_ecosystems
            .iter()
            .any(|exempt| ecosystem == Some(exempt.as_str()))
        {
            continue;
        }
        let update = Update {
            file,
            path: ConfigPath::default().key("updates").index(index),
            label: format!(
                "Dependabot update #{} ({})",
                index + 1,
                ecosystem.unwrap_or("unknown ecosystem")
            ),
            locations,
            message,
        };
        let cooldown = entry.get("cooldown");
        for key in DAYS {
            let days = cooldown.and_then(|value| value.get(key));
            if (key == "default-days" || days.is_some())
                && !days
                    .and_then(Value::as_u64)
                    .is_some_and(|days| days >= minimum_days)
            {
                let target = format!("cooldown.{key}");
                findings.push(update.finding(
                    update.path.clone().key("cooldown").key(key),
                    &target,
                    format!("set {target} to an integer of at least {minimum_days}"),
                ));
            }
        }
        if options.require_open_pull_requests_limit
            && entry.get(PR_LIMIT).and_then(Value::as_u64).is_none()
        {
            findings.push(update.finding(
                update.path.clone().key(PR_LIMIT),
                PR_LIMIT,
                format!("set {PR_LIMIT} to a nonnegative integer"),
            ));
        }
        if let (Some(allowed), Some(excludes)) =
            (allowed, cooldown.and_then(|value| value.get("exclude")))
        {
            update.check_excludes(allowed, excludes, &mut findings);
        }
    }
    findings
}

struct Update<'a> {
    file: &'a str,
    path: ConfigPath,
    label: String,
    locations: Option<&'a ConfigLocations>,
    message: Option<&'a str>,
}

impl Update<'_> {
    fn check_excludes(&self, allowed: &GlobSet, excludes: &Value, findings: &mut Vec<RuleFinding>) {
        let target = "cooldown.exclude";
        let path = self.path.clone().key("cooldown").key("exclude");
        let Some(entries) = excludes.as_sequence() else {
            findings.push(self.finding(
                path,
                target,
                format!("set {target} to a list of allowed package patterns"),
            ));
            return;
        };
        for (index, entry) in entries.iter().enumerate() {
            if let Some(pattern) = entry.as_str() {
                if !allowed.is_match(pattern) {
                    findings.push(self.finding(
                        path.clone().index(index),
                        target,
                        format!("remove `{pattern}` from {target} or match it with options.allowedCooldownExcludes"),
                    ));
                }
            } else {
                findings.push(self.finding(
                    path.clone().index(index),
                    target,
                    format!(
                        "set {target} entry #{} to an allowed package pattern",
                        index + 1
                    ),
                ));
            }
        }
    }

    fn finding(&self, mut path: ConfigPath, target: &str, fix: String) -> RuleFinding {
        let line = self.locations.map_or(1, |locations| loop {
            if let Some(line) = locations.line(&path) {
                break line;
            }
            if path.pop().is_none() {
                break 1;
            }
        });
        let prefix = self
            .message
            .map(|message| format!("{message} "))
            .unwrap_or_default();
        let message = format!("{prefix}{}: {fix}.", self.label);
        RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_owned(),
            file: self.file.to_owned(),
            line,
            message,
            import: None,
            target: Some(target.to_owned()),
        }
    }
}
