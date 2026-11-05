use super::PreparedCheckInputs;
use no_mistakes::codebase::analysis_session::AnalysisSession;
pub(crate) fn record(session: &AnalysisSession, prepared: &PreparedCheckInputs) {
    if let Some(playwright) = &prepared.playwright {
        for failure in playwright.failures() {
            session.record_check_failure(failure.clone());
        }
    }
    if let Some(message) = &prepared.playwright_failure {
        for consumer in [
            "playwright-coverage",
            "playwright-unique-test-ids",
            "playwright-unique-html-ids",
            "playwright-prefer-test-id-locators",
        ] {
            if prepared.config.rule_configured(consumer) {
                session.record_check_failure(no_mistakes::codebase::check_outcome::CheckFailure {
                    consumer: consumer.to_string(),
                    file: None,
                    property: None,
                    message: message.clone(),
                });
            }
        }
    }
}
