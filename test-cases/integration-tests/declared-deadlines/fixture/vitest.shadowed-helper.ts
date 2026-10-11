import { defineConfig } from 'vitest/config';
function options(defineConfig = (_value: unknown) => ({testTimeout:30001})) {
  const config = defineConfig({testTimeout:1});
  return config;
}
export default defineConfig({test:{...options(),hookTimeout:1}});
