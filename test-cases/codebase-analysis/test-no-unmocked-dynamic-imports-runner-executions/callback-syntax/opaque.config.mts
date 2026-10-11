import { defineConfig } from 'vitest/config';

// Either branch can execute; syntax from one branch cannot grant global mocks.
export default defineConfig(() => globalThis.choose
  ? { test: { name: 'opaque', include: ['tests/opaque.test.mts'], setupFiles: ['./comment.setup.mts'] } }
  : { test: { name: 'opaque', include: ['tests/opaque.test.mts'] } });
