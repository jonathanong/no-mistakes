import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { __dirname } from "./helpers.mjs";

export const RULE = "require-options-on-imported-call";

export const ssrfOptions = {
  targets: [
    {
      sourceSpecifierPatterns: ["ssrf-guard/node"],
      calleeNamePatterns: ["validateUrl"],
      optionsPosition: 2,
      requiredProperties: ["timeoutMs", "signal"],
    },
  ],
};

export function ruleFixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", name),
    "utf8",
  );
}
