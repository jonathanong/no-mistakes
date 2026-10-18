// `process.env` stays dynamic on purpose. A string literal would resolve the
// setup and stop the owning-project fallback this fixture protects.
const dynamicAppSetup = process.env.FIXTURE_APP_SETUP;

export default {
  test: {
    projects: [
      {
        test: {
          name: "app",
          include: ["app/**/*.test.ts"],
          setupFiles: dynamicAppSetup,
        },
      },
      {
        test: {
          name: "tooling",
          include: ["tooling/**/*.test.ts"],
        },
      },
    ],
  },
};
