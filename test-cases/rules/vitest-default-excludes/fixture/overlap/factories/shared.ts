import { configDefaults } from "vitest/config";

export function makeVitestProject() {
  return {
    test: {
      exclude: ["**/dist/**"],
    },
  };
}

export function makeSafeVitestProject() {
  return {
    test: {
      exclude: [...configDefaults.exclude, "**/dist/**"],
    },
  };
}
