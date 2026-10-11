import { defineConfig } from 'vitest/config';
const cfg = defineConfig(() => ({test:{name:'named',include:['tests/named*.test.mts'],setupFiles:['./named.setup.mts']}}));
const alias = cfg;
export { alias as default };
