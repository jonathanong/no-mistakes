import { defineConfig } from 'vitest/config';
export default defineConfig(() => ({test:{name:'spread-root',include:['tests/spread-root.test.mts'],setupFiles:['./old.setup.mts']}, ...globalThis.options}));
