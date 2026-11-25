import { defineConfig } from "vitest/config";
import { changedPatterns, extra as imported, gitMetadata } from "./excludes";
import { selectedHelper } from "./vitest-shared.config";

export default defineConfig({
  test: {
    exclude: ["**/node_modules/**", ...imported],
    projects: [
      { test: { exclude: ["**/node_modules/**", ...gitMetadata] } },
      { test: { exclude: ["**/node_modules/**", ...selectedHelper] } },
      { test: { exclude: ["**/node_modules/**", ...changedPatterns] } },
    ],
  },
});
