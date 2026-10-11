function mergeConfig(first: unknown, second: unknown) { return second; }
const config = mergeConfig({ test: { testTimeout: 1 } }, { test: { testTimeout: 30001 } });
export default config;
