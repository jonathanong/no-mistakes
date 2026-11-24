import {
  analyzeProject,
  check,
  type CheckReport,
  type PlaywrightFixtureExport,
  type PlaywrightNoSerialOptions,
  type TestNoRetriesOptions,
  type TestReliabilityAllowlistEntry,
} from "no-mistakes";

const externalSuite: TestReliabilityAllowlistEntry = {
  path: "tests/external/**",
  reason: "Runs a browser on a remote test grid",
};
const wrappedFixture: PlaywrightFixtureExport = {
  module: "helpers/test.mts",
  export: "test",
};
export const retries: TestNoRetriesOptions = {
  maxRetries: 1,
  testCommands: [["pnpm", "test:unit"]],
  files: ["config/browser-options.ts"],
  fixtures: [wrappedFixture],
  allowlist: [externalSuite],
};
export const serial: PlaywrightNoSerialOptions = {
  fixtures: [wrappedFixture],
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
