This saved fixture has a dedicated root for the single analyzeProject helper-recovery
parse-count test. Its physical copy prevents parallel model and policy tests from
contributing to the request-wide counter. Keep this root exclusive to that test.
