use crate::queue::graph_model::{CheckFinding, InternalProducer, InternalWorker};
use crate::queue::source::relative_string;
use crate::queue::types::{JobKey, QueueProducer, QueueWorker};
use std::path::Path;

impl InternalProducer {
    pub(super) fn job_key(&self) -> Option<(JobKey, &Self)> {
        let queue = self.queue.as_ref()?;
        Some((
            JobKey {
                queue_file: queue.queue_file.clone(),
                queue_name: queue.queue_name.clone(),
                job: self.site.job.clone()?,
            },
            self,
        ))
    }

    pub(super) fn public(&self, root: &Path) -> QueueProducer {
        QueueProducer {
            file: relative_string(root, &self.site.file),
            line: self.site.line,
            queue_file: self
                .queue
                .as_ref()
                .map(|q| relative_string(root, &q.queue_file)),
            queue_name: self.queue.as_ref().map(|q| q.queue_name.clone()),
            job: self.site.job.clone(),
            raw_job: self.site.raw_job.clone(),
            library: None,
        }
    }

    pub(super) fn unmatched(&self, root: &Path) -> CheckFinding {
        CheckFinding {
            kind: "unmatched-producer".to_string(),
            file: relative_string(root, &self.site.file),
            line: self.site.line,
            queue_file: self
                .queue
                .as_ref()
                .map(|q| relative_string(root, &q.queue_file)),
            queue_name: self.queue.as_ref().map(|q| q.queue_name.clone()),
            job: self.site.job.clone(),
            message: "static queue producer has no matching worker".to_string(),
        }
    }
}

impl InternalWorker {
    pub(super) fn job_keys(&self) -> Vec<(JobKey, &Self)> {
        let Some(queue) = &self.queue else {
            return Vec::new();
        };
        self.site
            .jobs
            .iter()
            .map(|job| {
                (
                    JobKey {
                        queue_file: queue.queue_file.clone(),
                        queue_name: queue.queue_name.clone(),
                        job: job.clone(),
                    },
                    self,
                )
            })
            .collect()
    }

    pub(super) fn public(&self, root: &Path) -> QueueWorker {
        QueueWorker {
            file: relative_string(root, &self.site.file),
            line: self.site.line,
            processor_file: self
                .site
                .processor_file
                .as_ref()
                .map(|p| relative_string(root, p)),
            queue_file: self
                .queue
                .as_ref()
                .map(|q| relative_string(root, &q.queue_file)),
            queue_name: self.queue.as_ref().map(|q| q.queue_name.clone()),
            jobs: self.site.jobs.clone(),
            wildcard: self.site.wildcard,
            library: None,
        }
    }

    pub(super) fn unmatched(&self, root: &Path, job: &JobKey) -> CheckFinding {
        CheckFinding {
            kind: "unmatched-worker".to_string(),
            file: relative_string(root, &self.site.file),
            line: self.site.line,
            queue_file: Some(relative_string(root, &job.queue_file)),
            queue_name: Some(job.queue_name.clone()),
            job: Some(job.job.clone()),
            message: "static queue worker has no matching producer".to_string(),
        }
    }
}
