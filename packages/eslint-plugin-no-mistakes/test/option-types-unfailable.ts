import type {
  TestNoUnfailableAssertionCode,
  TestNoUnfailableAssertionOptions,
  TestNoUnfailableAssertionReport,
  TestNoUnfailableAssertionSeverity,
} from "../src/index.js";
import type pluginRequire = require("../src/index.js");

export const code: TestNoUnfailableAssertionCode = "throwing-query";
export const severity: TestNoUnfailableAssertionSeverity = "error";
export const report: TestNoUnfailableAssertionReport = "error";
export const unfailableOptions: TestNoUnfailableAssertionOptions = {
  severities: { [code]: severity, "defined-then-deref": "off" },
  report,
};
export const defaultUnfailableOptions: pluginRequire.TestNoUnfailableAssertionOptions = {};
// @ts-expect-error Severity must be error, advisory, or off.
export const invalidSeverity: TestNoUnfailableAssertionSeverity = "warn";
// @ts-expect-error Report filter must be all, error, or advisory.
export const invalidReport: TestNoUnfailableAssertionReport = "warn";
