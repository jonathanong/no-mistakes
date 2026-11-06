import {
  analyzeProject,
  check,
  type CheckReport,
  type VitestProjectPolicyOptions,
  type VitestPropertyLiteral,
  type VitestPropertyParity,
} from "no-mistakes";

const nullable: VitestPropertyLiteral = null;
const pair: VitestPropertyParity = {
  properties: ["fileParallelism", "maxWorkers"],
  compareRoot: true,
};
export const policy: VitestProjectPolicyOptions = {
  config: "vitest.config.ts",
  helpers: ["config/test-projects.ts"],
  required: { isolate: true, nullable },
  forbidden: ["poolOptions"],
  parity: [pair],
};
export const report: Promise<CheckReport> = check({ root: "." });
export const batch = analyzeProject({ root: ".", reports: [{ type: "check" }] });
