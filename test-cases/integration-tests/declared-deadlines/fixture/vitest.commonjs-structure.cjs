// Existing CommonJS discovery is retained; this is not public SDK deadline proof.
module.exports = {
  test: {
    root: "./legacy-root",
    include: ["owned/*.test.ts"],
    setupFiles: "./legacy-setup.ts",
    testTimeout: 30001,
  },
};
