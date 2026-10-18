use super::findings::{finding, incomplete, unused};
use super::resolve::Resolver;
use super::CompiledInventory;
use crate::codebase::ci_workflows::{secret_facts::Environment, PreparedWorkflowTopologyInputs};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::codebase::workflow_topology::model::{WorkflowCallSecretsBinding, WorkflowTopologyEdge};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(in super::super) fn lint(
    root: &Path,
    inputs: &PreparedWorkflowTopologyInputs,
    sources: &SourceStore,
    inventory: &CompiledInventory,
    config_path: Option<&Path>,
) -> Vec<RuleFinding> {
    let documents = inputs.secret_documents(sources);
    let topology = inputs.topology();
    let workflows: BTreeMap<_, _> = topology
        .workflows
        .iter()
        .map(|node| (node.path.as_str(), node))
        .collect();
    let mut findings = Vec::new();
    let mut used = BTreeSet::new();
    let mut complete = true;
    for (path, document) in &documents.workflows {
        for (line, reason) in &document.incomplete {
            incomplete(&mut findings, path, *line, reason);
        }
        complete &= document.incomplete.is_empty();
        let workflow = workflows.get(path.as_str());
        let aliases: BTreeSet<_> = workflow
            .and_then(|node| node.workflow_call.as_ref())
            .map(|contract| {
                contract
                    .secrets
                    .keys()
                    .map(|name| name.to_ascii_lowercase())
                    .collect()
            })
            .unwrap_or_default();
        let reusable = workflow.is_some_and(|node| node.callable);
        if reusable
            && !aliases.is_empty()
            && workflow.is_some_and(|node| {
                node.triggers
                    .iter()
                    .any(|trigger| trigger.event != "workflow_call")
            })
        {
            complete = false;
            incomplete(
                &mut findings,
                path,
                1,
                "reusable contract also has direct triggers; secret sources depend on the trigger",
            );
        }
        let mut resolver = Resolver {
            inventory,
            used: &mut used,
            findings: &mut findings,
        };
        for reference in &document.references {
            complete &= resolver.reference(
                path,
                reference,
                &Environment::Repository,
                &aliases,
                reusable,
            );
        }
        for scope in document.jobs.values() {
            for reference in &scope.references {
                complete &=
                    resolver.reference(path, reference, &scope.environment, &aliases, reusable);
            }
        }
    }
    for edge in &topology.edges {
        let WorkflowTopologyEdge::Calls(call) = edge else {
            continue;
        };
        let (path, key) = call
            .from
            .rsplit_once('#')
            .expect("canonical job IDs contain a separator");
        let line = documents
            .workflows
            .get(path)
            .and_then(|document| document.jobs.get(key))
            .map_or(1, |scope| scope.line);
        let unresolved = !call.local
            || call
                .to
                .as_deref()
                .is_none_or(|target| workflows.get(target).is_none_or(|node| !node.callable));
        let inherited = matches!(call.bindings.secrets, WorkflowCallSecretsBinding::Inherit);
        if unresolved || inherited {
            complete = false;
            incomplete(
                &mut findings,
                path,
                line,
                if inherited {
                    "reusable call inherits secrets"
                } else {
                    "reusable call cannot be resolved locally"
                },
            );
        }
    }
    for diagnostic in &topology.diagnostics {
        if diagnostic.code
            == crate::codebase::workflow_topology::model::DiagnosticCode::WorkflowCallCycle
        {
            complete = false;
            incomplete(
                &mut findings,
                &diagnostic.workflow_path,
                1,
                "reusable call cycle",
            );
        }
    }
    for (path, document) in &documents.actions {
        for (line, reason) in &document.incomplete {
            incomplete(&mut findings, path, *line, reason);
        }
        complete &= document.incomplete.is_empty();
        if document.composite {
            for reference in &document.references {
                findings.push(finding(path, reference.line, "composite actions cannot access the secrets context directly; pass the value explicitly through inputs or env".into()));
                complete = false;
            }
        }
    }
    if complete && inventory.report_unused {
        unused(
            root,
            inputs,
            config_path,
            sources,
            inventory,
            &used,
            &mut findings,
        );
    }
    findings
}
