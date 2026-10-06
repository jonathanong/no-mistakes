import { defineConfig as config, defineProject as project } from 'vitest/config';
const LARGE = 60_000;
const reusable = { test: { testTimeout: LARGE } };
export default config({
  test: {
    testTimeout: LARGE,
    hookTimeout: 5_000,
    projects: [
      { test: { testTimeout: 60_000 } },
      project({ test: { hookTimeout: 60_000 } }),
      reusable,
      { test: { testTimeout: 5_000, hookTimeout: 5_000 } },
      'other.project.ts',
    ],
  },
});
