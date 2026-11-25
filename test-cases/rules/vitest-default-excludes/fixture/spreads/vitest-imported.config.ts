import { defineConfig } from "vitest/config";
import { extra as imported, gitMetadata } from "./excludes";

export default defineConfig({
  test: {
    exclude: ["**/node_modules/**", ...imported],
    projects: [{ test: { exclude: ["**/node_modules/**", ...gitMetadata] } }],
  },
});
