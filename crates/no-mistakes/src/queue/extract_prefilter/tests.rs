use super::{contains_bytes, source_may_have_queue_facts};
use crate::config::v2::schema::{QueueFactoryRoles, QueueNameFactory};

#[test]
fn empty_factory_name_disables_the_prefilter() {
    assert!(source_may_have_queue_facts(
        "export const value = 1;",
        &[String::new()],
        &QueueFactoryRoles::default(),
    ));
}

#[test]
fn package_and_factory_needles_are_required_substrings() {
    let mut roles = QueueFactoryRoles::default();
    roles.queue.push(QueueNameFactory {
        callee: "createQueue".to_string(),
        name: 0,
    });
    assert!(source_may_have_queue_facts(
        "import { Queue } from 'bullmq';",
        &[],
        &QueueFactoryRoles::default(),
    ));
    assert!(source_may_have_queue_facts(
        "export const queue = createQueue('mail');",
        &["createQueue".to_string()],
        &roles,
    ));
    assert!(!source_may_have_queue_facts(
        "export const JOB = 'resize';",
        &["createQueue".to_string()],
        &roles,
    ));
    assert!(source_may_have_queue_facts(
        "ns.q.add(\"job\", {});",
        &["createQueue".to_string()],
        &roles,
    ));
}

#[test]
fn memmem_finds_ascii_needles() {
    assert!(contains_bytes(b"import 'bullmq'", b"bullmq"));
    assert!(!contains_bytes(b"import 'bull'", b"bullmq"));
}
