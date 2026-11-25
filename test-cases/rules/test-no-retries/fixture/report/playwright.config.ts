import { defineConfig as browserConfig } from "@playwright/test";

export default browserConfig({
  retries: 1,
  workers: 1,
  fullyParallel: false,
  testDir: "./tests",
  projects: [
    { name: "approved", retries: 3, workers: 1, fullyParallel: false },
    { name: "browser", retries: 2, workers: 1, fullyParallel: false },
    { name: "parallel", retries: 0, workers: 4, fullyParallel: true },
  ],
});
