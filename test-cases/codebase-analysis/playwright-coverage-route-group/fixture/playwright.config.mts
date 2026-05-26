import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "tests",
  use: {
    testIdAttribute: "data-pw",
  },
});
