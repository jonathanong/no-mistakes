import * as v from 'vitest/config';
const LARGE = 60_000;
const high = { testTimeout: LARGE, hookTimeout: LARGE };
const low = { testTimeout: 5000, hookTimeout: 5000 };
export default v.defineConfig({ test: { ...high, ...low, testTimeout: LARGE, testTimeout: 5000, projects: [{ test: { ...high, hookTimeout: 5000 } }] } });
