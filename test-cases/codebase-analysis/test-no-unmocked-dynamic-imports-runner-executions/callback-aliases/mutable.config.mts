import { defineConfig } from 'vitest/config';
let cfg = defineConfig(() => ({test:{name:'mutable',include:['tests/mutable.test.mts'],setupFiles:['./old.setup.mts']}}));
cfg = defineConfig(() => ({test:{name:'mutable',include:['tests/mutable.test.mts']}}));
export default cfg;
