import { defineConfig as config } from 'vitest/config';

export default config(function () {
  return { test: { name: 'web', root: './web', setupFiles: ['./setup.mts'] } };
});
