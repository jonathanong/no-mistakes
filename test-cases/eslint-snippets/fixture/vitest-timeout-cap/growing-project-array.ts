import { defineConfig, mergeConfig } from "vitest/config";
const grew = [];
grew.push({ test: { testTimeout: 30001 } });
export default mergeConfig({ test: { projects: grew } }, { test: { projects: [] } });
