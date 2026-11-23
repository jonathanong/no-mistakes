use super::{Name, Queue, Target, WrapperSite};
use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::queue::extract_model::{FileFacts, ProducerBinding, ProducerSite};
use crate::queue::factory_resolve::follow::{lookup, Want};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(in crate::queue) fn resolve<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
) {
    let extracted = facts
        .iter_mut()
        .filter(|(_, file)| !file.enqueue_wrappers.is_empty())
        .map(|(path, file)| (path.clone(), std::mem::take(&mut file.enqueue_wrappers)))
        .collect::<Vec<_>>();
    for (path, wrappers) in extracted {
        for wrapper in wrappers {
            let (binding, job) = target(facts, resolver, remapper, &path, &wrapper.target);
            let file = facts
                .get_mut(&path)
                .expect("wrapper source has queue facts");
            publish(file, &path, wrapper, binding, job);
        }
    }
}

fn target<R: ImportResolverFacade>(
    facts: &HashMap<PathBuf, FileFacts>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
    path: &Path,
    target: &Target,
) -> (ProducerBinding, Option<String>) {
    let resolve_name = |name: &Name, stable: Want<'_>| match name {
        Name::Literal(value) => Some(value.clone()),
        Name::Import { local, property } => {
            let want = property.as_deref().map_or(stable, Want::Property);
            lookup(facts, resolver, remapper, path, local, want)
        }
    };
    let binding = match &target.queue {
        Queue::Unknown => ProducerBinding::Unknown,
        Queue::Name(name) => resolve_name(name, Want::StableName)
            .map_or(ProducerBinding::Unknown, ProducerBinding::QueueName),
        Queue::Binding(binding) => binding.clone(),
    };
    (
        binding,
        target
            .job
            .as_ref()
            .and_then(|name| resolve_name(name, Want::StableString)),
    )
}

fn publish(
    file: &mut FileFacts,
    path: &Path,
    mut wrapper: WrapperSite,
    binding: ProducerBinding,
    job: Option<String>,
) {
    let self_dedup = job.as_ref().is_some_and(|job| {
        wrapper.dedup && wrapper.current_case.as_ref().is_none_or(|case| case == job)
    });
    file.producers.push(ProducerSite {
        file: path.to_path_buf(),
        line: wrapper.options.line,
        binding: binding.clone(),
        job: job.clone(),
        job_import: None,
        raw_job: Some(wrapper.raw_job),
        worker_index: wrapper.worker_index,
        self_dedup,
        priority: wrapper.priority,
    });
    wrapper.options.binding = Some(binding.clone());
    wrapper.options.job = job.clone();
    file.job_option_sites.push(wrapper.options);
    for mut check in wrapper.checks {
        check.job = job.clone();
        file.job_id_sites.push((
            crate::queue::extract_model::JobIdQueue::Named(binding.clone()),
            check,
        ));
    }
}
