use super::{apply_update, Update};
use crate::queue::extract_model::{FileFacts, ProducerSite, WorkerSite};
use std::collections::HashMap;
use std::path::PathBuf;

fn path() -> PathBuf {
    PathBuf::from("queue.mts")
}

fn worker(queue_name: Option<&str>) -> WorkerSite {
    WorkerSite {
        file: path(),
        line: 1,
        queue_name: queue_name.map(str::to_string),
        queue_name_import: None,
        jobs: Vec::new(),
        processor_specifier: None,
        processor_file: None,
        wildcard: true,
    }
}

fn producer(job: Option<&str>) -> ProducerSite {
    ProducerSite {
        file: path(),
        line: 1,
        binding: "q".to_string(),
        job: job.map(str::to_string),
        job_import: None,
        raw_job: None,
        worker_index: None,
        self_dedup: false,
    }
}

#[test]
fn stale_updates_do_not_overwrite_resolved_names() {
    let mut facts = HashMap::new();
    let mut file = FileFacts::default();
    file.queue_bindings
        .insert("q".to_string(), "old".to_string());
    file.queue_exports
        .insert("q".to_string(), "old".to_string());
    file.workers.push(worker(Some("named")));
    file.producers.push(producer(Some("kept")));
    facts.insert(path(), file);
    let missing = PathBuf::from("missing.mts");

    assert!(!apply_update(
        &mut facts,
        Update::Binding {
            file: missing.clone(),
            binding: "q".into(),
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::Export {
            file: missing.clone(),
            exported: "q".into(),
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::Worker {
            file: missing.clone(),
            index: 0,
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::Worker {
            file: path(),
            index: 3,
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::Worker {
            file: path(),
            index: 0,
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::ProducerJob {
            file: missing,
            index: 0,
            job: "other".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::ProducerJob {
            file: path(),
            index: 3,
            job: "other".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::ProducerJob {
            file: path(),
            index: 0,
            job: "other".into(),
        }
    ));

    let file = &facts[&path()];
    assert_eq!(file.queue_bindings["q"], "old");
    assert_eq!(file.queue_exports["q"], "old");
    assert_eq!(file.workers[0].queue_name.as_deref(), Some("named"));
    assert_eq!(file.producers[0].job.as_deref(), Some("kept"));
    assert!(!apply_update(
        &mut facts,
        Update::Binding {
            file: path(),
            binding: "q".into(),
            queue_name: "new".into(),
        }
    ));
    assert!(!apply_update(
        &mut facts,
        Update::Export {
            file: path(),
            exported: "q".into(),
            queue_name: "new".into(),
        }
    ));
}
