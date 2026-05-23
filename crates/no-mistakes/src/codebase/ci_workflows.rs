use anyhow::Result;

mod cargo;

pub use cargo::{
    extract_binary_names, extract_cargo_targets, parse_cargo_bins, parse_cargo_package_name,
    parse_cargo_workspace_excludes, parse_cargo_workspace_members, CargoTarget,
};

/// A single step invocation extracted from a CI workflow.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    /// Optional step name (from `name:` field).
    pub step_name: Option<String>,
    /// Raw `run:` string or `uses:` string.
    pub run: String,
    /// 1-based line number (approximate; YAML parsing loses exact line info).
    pub line: u32,
    /// Binary names extracted from the run string.
    pub binaries: Vec<String>,
    pub cargo_targets: Vec<CargoTarget>,
}

/// Extract all `run:` step invocations from a GitHub Actions workflow YAML.
pub fn extract_invocations(workflow_yaml: &str) -> Result<Vec<Invocation>> {
    let value: serde_yaml::Value = serde_yaml::from_str(workflow_yaml)?;
    let mut results = Vec::new();
    let mut line_counter: u32 = 1;

    if let Some(jobs) = value.get("jobs").and_then(|v| v.as_mapping()) {
        for (_job_id, job) in jobs {
            if let Some(steps) = job.get("steps").and_then(|v| v.as_sequence()) {
                for step in steps {
                    let step_name = step
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);

                    if let Some(run_str) = step.get("run").and_then(|v| v.as_str()) {
                        let binaries = extract_binary_names(run_str);
                        let cargo_targets = extract_cargo_targets(run_str);
                        results.push(Invocation {
                            step_name,
                            run: run_str.to_string(),
                            line: line_counter,
                            binaries,
                            cargo_targets,
                        });
                        line_counter += run_str.lines().count() as u32 + 1;
                    } else if let Some(uses_str) = step.get("uses").and_then(|v| v.as_str()) {
                        results.push(Invocation {
                            step_name,
                            run: uses_str.to_string(),
                            line: line_counter,
                            binaries: vec![],
                            cargo_targets: vec![],
                        });
                        line_counter += 1;
                    }
                }
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests;
