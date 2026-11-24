import type { PlaywrightNoSerialAllowEntry, PlaywrightNoSerialOptions } from "no-mistakes";

export const entry: PlaywrightNoSerialAllowEntry = {
  path: "e2e/legacy/*.spec.ts",
  project: "legacy",
  reason: "serial fixture migration in progress",
};

export const options: PlaywrightNoSerialOptions = {
  allow: [entry],
  files: ["e2e/**/*.ts", "playwright.config.ts"],
  configs: ["configs/playwright.ts"],
};

// @ts-expect-error Allow entries require a reason.
export const missingReason: PlaywrightNoSerialAllowEntry = {
  project: "legacy",
};
