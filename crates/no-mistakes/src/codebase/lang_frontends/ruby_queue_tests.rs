use super::*;
use crate::codebase::lang_frontends::strip::strip_comments_keep_strings;

#[test]
fn perform_later_and_perform_async_extract_job_classes() {
    let source = r#"
WelcomeJob.perform_later(user)
MailWorker.perform_async(user)
Workers::DigestJob.perform_async
"#;
    let enqueues = extract_enqueues(source);
    assert!(enqueues.iter().any(|name| name == "WelcomeJob"));
    assert!(enqueues.iter().any(|name| name == "MailWorker"));
    assert!(enqueues.iter().any(|name| name == "DigestJob"));
    assert!(!enqueues.iter().any(|name| name == "Workers::DigestJob"));
}

#[test]
fn application_job_and_sidekiq_includes_are_workers() {
    let source = r#"
class WelcomeJob < ApplicationJob
end

class MailWorker
  include Sidekiq::Worker
end

class DigestJob
  include Sidekiq::Job
end
"#;
    let workers = extract_workers(source);
    assert!(workers.iter().any(|name| name == "WelcomeJob"));
    assert!(workers.iter().any(|name| name == "MailWorker"));
    assert!(workers.iter().any(|name| name == "DigestJob"));
}

#[test]
fn compact_class_include_and_namespaced_class_share_short_identity() {
    let workers = extract_workers(
        "class MailWorker; include Sidekiq::Worker; end\nclass Workers::DigestJob\n  include Sidekiq::Job\nend\n",
    );
    assert!(workers.iter().any(|name| name == "MailWorker"));
    assert!(workers.iter().any(|name| name == "DigestJob"));
    assert!(!workers.iter().any(|name| name == "Workers::DigestJob"));
}

#[test]
fn string_literal_enqueue_examples_are_not_jobs() {
    let source = strip_comments_keep_strings(
        r#"
logger.info("MailWorker.perform_async failed")
WelcomeJob.perform_later(user)
"#,
    );
    assert_eq!(extract_enqueues(&source), vec!["WelcomeJob".to_string()]);
}

#[test]
fn string_literal_include_is_not_a_worker() {
    let source = strip_comments_keep_strings(
        r#"
class Other
  logger.info("include Sidekiq::Worker")
end
"#,
    );
    assert!(extract_workers(&source).is_empty());
}

#[test]
fn nested_class_include_binds_outer_worker() {
    let workers = extract_workers(
        r#"
class BillingWorker
  class Helper
  end
  include Sidekiq::Worker
end
"#,
    );
    assert_eq!(workers, vec!["BillingWorker".to_string()]);
}

#[test]
fn computed_enqueue_and_unrelated_class_are_non_edges() {
    let source = r#"
class Other
end

kls = const_get(name)
kls.perform_async
name.constantize.perform_later
"#;
    assert!(extract_enqueues(source).is_empty());
    assert!(extract_workers(source).is_empty());
}

#[test]
fn comment_enqueue_examples_are_not_jobs() {
    let source = strip_comments_keep_strings(
        r#"
# MailWorker.perform_async(user)
WelcomeJob.perform_later(user)
"#,
    );
    assert_eq!(extract_enqueues(&source), vec!["WelcomeJob".to_string()]);
}
