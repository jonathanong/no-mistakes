import * as vitest from 'vitest/config';

export default vitest.defineConfig(() => ({
  test: {
    projects: [
      { test: { name: 'a', include: ['tests/multi.test.mts'], setupFiles: ['./multi.setup.mts'] } },
      { test: { name: 'b', include: ['tests/multi.test.mts'] } },
    ],
  },
}));
