export default {
  testDir: "./e2e",
  testMatch: ["**/*.spec.ts"],
  projects: [
    { name: "chromium", testDir: "./e2e", testMatch: ["**/*.spec.ts"] },
  ],
};
