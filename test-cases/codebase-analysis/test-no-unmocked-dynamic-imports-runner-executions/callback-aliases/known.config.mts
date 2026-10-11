import { defineConfig } from 'vitest/config';
export default defineConfig(() => ({test:{...{name:'known',include:['tests/known.test.mts'],setupFiles:['./known.setup.mts']}}}));
