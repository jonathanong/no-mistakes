// no-mistakes-disable-file postgres-no-test-state-mutation
// no-mistakes-disable-file forbidden-lint-suppressions
// no-mistakes-disable-file typo-configured-as-a-rule
// no-mistakes-disable-file postgres-require-fk-on-delete
// no-mistakes-disable-next-line postgres-no-test-state-mutation
export const sql = 'DELETE FROM test_state'; // File directives intentionally shadow the next-line directive.
