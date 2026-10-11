use super::*;
use crate::codebase::ts_source::relative_slash_path;
use crate::integration_tests::types::*;

impl PreparedIntegrationRunnerConfigs {
    /// Project retained results only. This function performs no reads or parsing.
    #[doc(hidden)]
    pub fn deadline_evidence(
        &self,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Vec<RunnerConfigDeadlineEvidence> {
        [Framework::Playwright, Framework::Vitest]
            .into_iter()
            .map(|framework| {
                let mut specs = self
                    .specs
                    .iter()
                    .filter(|spec| spec.framework == framework)
                    .collect::<Vec<_>>();
                specs.sort_by(|a, b| a.raw.cmp(&b.raw));
                specs.dedup_by(|a, b| a.raw == b.raw);
                let configs = specs
                    .into_iter()
                    .map(|spec| {
                        let result =
                            facts
                                .integration_runner_configs
                                .get(&spec.path)
                                .and_then(|file| {
                                    file.results.iter().find(|result| {
                                        result.framework == framework && result.raw == spec.raw
                                    })
                                });
                        match result.map(|result| &result.projects) {
                            Some(Ok(projects)) => ConfigDeadlineEvidence::Prepared {
                                config: relative_slash_path(&self.root, &spec.path),
                                projects: projects
                                    .iter()
                                    .map(|project| project_evidence(&self.root, project))
                                    .collect(),
                            },
                            Some(Err(error)) => ConfigDeadlineEvidence::Failed {
                                config: relative_slash_path(&self.root, &spec.path),
                                error: error.clone(),
                            },
                            None => ConfigDeadlineEvidence::Failed {
                                config: relative_slash_path(&self.root, &spec.path),
                                error: "missing prepared runner config facts".to_string(),
                            },
                        }
                    })
                    .collect::<Vec<_>>();
                let requested = self.deadline_runners.contains(&framework.as_str());
                let status = if configs
                    .iter()
                    .any(|config| matches!(config, ConfigDeadlineEvidence::Failed { .. }))
                {
                    RunnerDeadlineStatus::Failed
                } else if requested || !configs.is_empty() {
                    RunnerDeadlineStatus::Prepared
                } else {
                    RunnerDeadlineStatus::NotRequested
                };
                RunnerConfigDeadlineEvidence {
                    framework: framework.as_str().to_string(),
                    status,
                    configs,
                }
            })
            .collect()
    }
}

pub(crate) fn project_evidence(
    root: &std::path::Path,
    project: &ConfigProject,
) -> ProjectDeadlineEvidence {
    ProjectDeadlineEvidence {
        config: project.config.as_ref().map(|path| {
            relative_slash_path(
                root,
                &crate::codebase::ts_resolver::normalize_path(&root.join(path)),
            )
        }),
        workspace: project.workspace,
        policy_name: project.policy_name.clone(),
        runner_project_arg: project.runner_project_arg.clone(),
        scope: project.scope.clone(),
        case: slot_evidence(root, project.declared_deadlines.case.as_ref()),
        hook: slot_evidence(root, project.declared_deadlines.hook.as_ref()),
        fixture: slot_evidence(root, project.declared_deadlines.fixture.as_ref()),
    }
}

fn slot_evidence(
    root: &std::path::Path,
    declaration: Option<&DeadlineDeclaration>,
) -> DeclaredDeadlineSlot {
    let Some(declaration) = declaration else {
        return DeclaredDeadlineSlot::Absent;
    };
    let provenance = DeadlineProvenance {
        path: relative_slash_path(root, &declaration.path),
        span: declaration.span,
        inherited_through: declaration
            .inherited_through
            .iter()
            .map(|via| DeadlineInheritanceEvidence {
                path: relative_slash_path(root, &via.path),
                project: via.project.clone(),
            })
            .collect(),
    };
    match declaration.value {
        DeadlineValue::Known(milliseconds) => DeclaredDeadlineSlot::Known {
            milliseconds,
            provenance,
        },
        DeadlineValue::Unknown(reason) => DeclaredDeadlineSlot::Unknown {
            reason: reason.into(),
            provenance,
        },
    }
}

impl From<DeadlineUnknownReason> for DeclaredDeadlineUnknownReason {
    fn from(reason: DeadlineUnknownReason) -> Self {
        match reason {
            DeadlineUnknownReason::Expression => Self::Expression,
            DeadlineUnknownReason::NonFinite => Self::NonFinite,
            DeadlineUnknownReason::OpaqueSpread => Self::OpaqueSpread,
            DeadlineUnknownReason::OpaqueTestObject => Self::OpaqueTestObject,
            DeadlineUnknownReason::ComputedProperty => Self::ComputedProperty,
            DeadlineUnknownReason::UnresolvedExtends => Self::UnresolvedExtends,
            DeadlineUnknownReason::UnresolvedInheritance => Self::UnresolvedInheritance,
            DeadlineUnknownReason::UnprovedConfigRoot => Self::UnprovedConfigRoot,
            DeadlineUnknownReason::Accessor => Self::Accessor,
            DeadlineUnknownReason::UnsupportedConfigCall => Self::UnsupportedConfigCall,
            DeadlineUnknownReason::UnprovedBinding => Self::UnprovedBinding,
        }
    }
}
