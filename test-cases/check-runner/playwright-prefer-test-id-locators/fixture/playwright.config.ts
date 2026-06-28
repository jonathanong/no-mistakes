export default {
  name: "web",
  testDir: "tests/e2e",
  use: {
    testIdAttribute: "data-pw",
  },
  projects: [{ name: "web" }],
};
