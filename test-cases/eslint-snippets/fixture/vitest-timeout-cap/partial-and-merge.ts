import { vi } from 'vitest';
import { defineConfig, mergeConfig } from 'vitest/config';
const setter = vi.setConfig.bind(vi, {testTimeout:60000});
setter({testTimeout:5000});
defineConfig(mergeConfig({}, importedConfig));
vi.setConfig({ ...opaque, testTimeout:30000, hookTimeout:30000 });
vi.setConfig({ ...opaque, testTimeout:30000 });
