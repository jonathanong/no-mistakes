import type { VitestDefaultExcludesOptions } from "no-mistakes";

export const defaults: VitestDefaultExcludesOptions = { config: "vitest.config.ts" };

export const custom: VitestDefaultExcludesOptions = {
  config: "vitest.config.ts",
  requiredGlobs: ["**/node_modules/**", "**/.git/**", "**/.sandbox/**"],
};
