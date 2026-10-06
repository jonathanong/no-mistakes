import { defineConfig, defineProject, mergeConfig } from 'vitest/config';
const LARGE = 60_000;
const base = defineConfig({ test: { testTimeout: LARGE, hookTimeout: LARGE, projects: [defineProject({ test: { testTimeout: LARGE } })] } });
const override = { test: { testTimeout: 5_000, hookTimeout: 5_000 } };
export default defineConfig(mergeConfig(base, override));
