import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    projects: [
      { test: { name: 'a', include: ['shared/*.test.mts'], exclude: ['shared/example.test.mts', 'shared/all-excluded.test.mts'], setupFiles: ['./setup/mock.mts'] } },
      { test: { name: 'b', include: ['shared/*.test.mts'], exclude: ['shared/all-excluded.test.mts'] } },
    ],
  },
});
