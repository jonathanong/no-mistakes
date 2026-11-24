import {
  analyzeProject,
  check,
  type CheckReport,
  type PlaywrightNoSerialOptions,
  type TestNoRetriesOptions,
  type TestReliabilityAllowlistEntry,
} from "no-mistakes";

const externalSuite: TestReliabilityAllowlistEntry = {
  path: "tests/external/**",
  reason: "Runs a browser on a remote test grid",
};
export const retries: TestNoRetriesOptions = {
  maxRetries: 1,
  testCommands: [["pnpm", "test:unit"]],
  files: ["config/browser-options.ts"],
  allowlist: [externalSuite],
};
export const serial: PlaywrightNoSerialOptions = {
  allowlist: [{ project: "migration", reason: "Exercises an ordered migration" }],
};
// @ts-expect-error Each exception requires a reason.
export const missingReason: TestReliabilityAllowlistEntry = { project: "migration" };
// @ts-expect-error Choose one selector per exception.
export const mixedSelectors: TestReliabilityAllowlistEntry = {
  ...externalSuite,
  project: "migration",
};

export const report: Promise<CheckReport> = check({ root: "." });
export const batch = analyzeProject({ root: ".", reports: [{ type: "check" }] });
