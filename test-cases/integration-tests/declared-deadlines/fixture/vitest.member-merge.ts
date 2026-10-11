// Preserve structural merge discovery without assuming a public SDK binding.
export default legacy.mergeConfig(
  { test: { root: "./before", testTimeout: 30001 } },
  {
    test: {
      root: "./legacy-root",
      include: ["owned/*.test.ts"],
      setupFiles: "./legacy-setup.ts",
      hookTimeout: 1,
    },
  },
);
