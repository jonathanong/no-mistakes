use super::*;

#[test]
fn php_requires_strip_to_file_stems() {
    let names = extract_php_requires(
        r#"require 'jobs/SendMail.php'; include_once(__DIR__ . '/Worker.php'); require_once 'dir/';"#,
    );
    assert!(names.contains(&"SendMail".to_string()));
    assert!(names.contains(&"Worker".to_string()));
}

#[test]
fn laravel_dispatches_resolve_aliases_and_namespaces() {
    let names = extract_laravel_dispatches(
        r#"
        namespace App\Jobs;
        use App\Mail\SendMail as MailJob;
        MailJob::dispatch();
        SendMail::dispatch();
        "#,
    );
    assert!(names.iter().any(|name| name.contains("SendMail")));
}

#[test]
fn laravel_queue_identities_prefer_qualified_names() {
    assert_eq!(
        laravel_queue_identities(&["SendMail".into()]),
        vec!["SendMail".to_string()]
    );
    assert_eq!(
        laravel_queue_identities(&["App.Jobs.SendMail".into(), "Other".into()]),
        vec!["App.Jobs.SendMail".to_string()]
    );
}

#[test]
fn messenger_extractors_cover_dispatch_handler_and_invoke_shapes() {
    let source = r#"
        dispatch(new App\Message\Ping());
        #[AsMessageHandler]
        final class PingHandler {}
        class OtherHandler implements MessageHandlerInterface {}
        function __invoke(Ping $message) {}
        "#;
    assert!(extract_messenger_dispatches(source)
        .iter()
        .any(|name| name.contains("Ping")));
    let workers = extract_messenger_workers(source);
    assert!(workers.iter().any(|name| name == "PingHandler"));
    assert!(workers.iter().any(|name| name == "OtherHandler"));
    assert!(php_should_queue_re().is_match("class Job implements ShouldQueue {}"));
}
