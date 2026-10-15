// Bare paths intentionally have no baseUrl: Vitest roots them at each project.
export default {
  test: {
    projects: [
      {
        root: "./web",
        test: {
          name: "web",
          include: ["**/*.test.ts"],
          setupFiles: ["setup/start.ts", "./setup/explicit.ts"],
          globalSetup: "setup/global.ts",
        },
      },
      {
        root: "./node",
        test: { name: "node", include: ["**/*.test.ts"], setupFiles: ["setup/start.ts"] },
      },
      {
        root: "./missing",
        test: { name: "missing", include: ["**/*.test.ts"], setupFiles: ["setup/deleted.ts"] },
      },
    ],
  },
};
