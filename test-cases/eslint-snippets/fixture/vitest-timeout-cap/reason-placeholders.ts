import { test } from "vitest";
// These templates are deliberately unfilled, even when copied from guidance.
// no-mistakes-timeout-reason: explanation
test("plain explanation", () => {}, 6000);
// no-mistakes-timeout-reason: Explanation
test("capitalized explanation", () => {}, 6000);
// no-mistakes-timeout-reason: <explanation>
test("bracketed explanation", () => {}, 6000);
// no-mistakes-timeout-reason: <EXPLANATION>
test("uppercase bracketed explanation", () => {}, 6000);
// no-mistakes-timeout-reason: specific reason
test("plain specific reason", () => {}, 6000);
// no-mistakes-timeout-reason: Specific Reason
test("capitalized specific reason", () => {}, 6000);
// no-mistakes-timeout-reason: <specific reason>
test("bracketed specific reason", () => {}, 6000);
// no-mistakes-timeout-reason: <Specific Reason>
test("capitalized bracketed specific reason", () => {}, 6000);
// no-mistakes-timeout-reason: explains the real worker provisioning delay
test("named worker setup", () => {}, 6000);
// no-mistakes-timeout-reason: waits for the specific worker process to exit
test("named worker shutdown", () => {}, 6000);
