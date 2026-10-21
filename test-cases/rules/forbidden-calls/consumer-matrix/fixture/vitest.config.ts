export default {
  test: {
    projects: [
      { test: { name: "unit", include: ["src/unit/**/*.test.mts"] } },
      {
        test: {
          name: "root-exclusion",
          include: ["src/root-exclusion/**/*.test.mts"],
        },
      },
      {
        test: {
          name: "integration",
          include: ["src/integration/**/*.test.mts"],
        },
      },
    ],
  },
};
