export default {
  testDir: "./e2e",
  use: { testIdAttribute: "data-pw" },
  projects: [
    {
      name: "control",
      testMatch: "control/**/*.spec.ts",
      use: { baseURL: "http://127.0.0.1:7421" },
    },
    {
      name: "agent",
      testMatch: "agent/**/*.spec.ts",
      use: { baseURL: "http://127.0.0.1:7422" },
    },
  ],
};
