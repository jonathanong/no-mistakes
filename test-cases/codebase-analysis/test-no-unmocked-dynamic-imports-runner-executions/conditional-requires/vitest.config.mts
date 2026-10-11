import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/*.test.cjs'],
    setupFiles: ['./setup.cjs'],
  },
});
