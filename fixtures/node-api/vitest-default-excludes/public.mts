import type { VitestDefaultExcludesOptions } from "no-mistakes";

export const defaults: VitestDefaultExcludesOptions = { config: "vitest.config.ts" };

export const factoriesOnly: VitestDefaultExcludesOptions = {
  files: ["config/vitest-*.ts"],
};

export const custom: VitestDefaultExcludesOptions = {
  config: "vitest.config.ts",
  files: ["config/vitest-*.ts"],
  requiredGlobs: ["**/node_modules/**", "**/.git/**", "**/.sandbox/**"],
};
