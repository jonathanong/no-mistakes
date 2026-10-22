use crate::config::v2::schema::QueueFactoryRoles;
use memchr::memmem;

pub(crate) fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    memmem::Finder::new(needle).find(haystack).is_some()
}

pub(crate) fn source_may_have_queue_facts(
    source: &str,
    factory_names: &[String],
    roles: &QueueFactoryRoles,
) -> bool {
    if factory_names.iter().any(|name| name.is_empty())
        || roles.queue.iter().any(|factory| factory.callee.is_empty())
        || roles
            .enqueue
            .iter()
            .any(|factory| factory.callee.is_empty())
        || roles.worker.iter().any(|factory| factory.callee.is_empty())
    {
        return true;
    }
    let haystack = source.as_bytes();
    contains_bytes(haystack, b"bullmq")
        || contains_bytes(haystack, b"glide-mq")
        || contains_bytes(haystack, b".add(")
        || contains_bytes(haystack, b".add (")
        || contains_bytes(haystack, b"addBulk")
        || contains_bytes(haystack, b"addAndWait")
        || contains_bytes(haystack, b"waitUntilFinished")
        || contains_bytes(haystack, b"new Queue")
        || contains_bytes(haystack, b"new Worker")
        || contains_bytes(haystack, b"FlowProducer")
        || contains_bytes(haystack, b"export *")
        || factory_names
            .iter()
            .any(|name| contains_bytes(haystack, name.as_bytes()))
        || roles
            .queue
            .iter()
            .any(|factory| contains_bytes(haystack, factory.callee.as_bytes()))
        || roles
            .enqueue
            .iter()
            .any(|factory| contains_bytes(haystack, factory.callee.as_bytes()))
        || roles
            .worker
            .iter()
            .any(|factory| contains_bytes(haystack, factory.callee.as_bytes()))
}

#[cfg(test)]
mod tests;
