import * as vite from "vite";
// Structural setup discovery is retained; mutable object identity is not deadline proof.
const shared = vite.defineConfig({
  test: {
    root: "./alias-root",
    include: ["owned/*.test.ts"],
    setupFiles: "./alias-setup.ts",
    testTimeout: 30001,
  },
});
export default vite.mergeConfig(shared, vite.defineConfig({ cacheDir: ".cache" }));
