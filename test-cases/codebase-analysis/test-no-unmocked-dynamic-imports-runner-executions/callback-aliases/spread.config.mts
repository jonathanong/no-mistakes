import { defineConfig } from 'vitest/config';
export default defineConfig(() => ({test:{name:'spread',include:['tests/spread.test.mts'],setupFiles:['./old.setup.mts'], ...globalThis.options}}));
