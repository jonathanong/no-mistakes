import type { TestNoRetriesAllowEntry, TestNoRetriesOptions } from "no-mistakes";

export const entry: TestNoRetriesAllowEntry = {
  path: "e2e/legacy/*.spec.ts",
  project: "smoke",
  reason: "shared staging sandbox rate limits",
};

export const options: TestNoRetriesOptions = {
  maxRetries: 0,
  allow: [entry],
  files: ["**/*.test.ts", ".github/workflows/*.yml"],
  configs: ["configs/vitest.ts"],
  retryActions: ["example-org/retry"],
  ciTestCommands: ["pnpm test"],
};

// @ts-expect-error Allow entries require a reason.
export const missingReason: TestNoRetriesAllowEntry = {
  path: "e2e/legacy/*.spec.ts",
};
