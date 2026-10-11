import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: { include: ['tests/covered.test.mts', 'tests/unmocked.test.mts'] },
});
