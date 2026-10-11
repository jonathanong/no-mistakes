import { defineConfig } from 'vitest/config';
declare const opaque: object;
export default defineConfig({ test: { ...opaque, projects: [
  { test: { name: 'local', testTimeout: 1 } },
] } });
