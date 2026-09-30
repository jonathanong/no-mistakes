// Not flagged by default: test-helpers/builders.ts is ordinary source to the
// built-in test classification. Flagged once `testFiles` covers test-helpers/**.
export class HelperOnlyError extends Error {}
