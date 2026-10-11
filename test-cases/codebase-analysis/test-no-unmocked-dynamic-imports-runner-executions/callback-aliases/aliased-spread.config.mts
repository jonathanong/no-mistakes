import { defineConfig } from 'vitest/config';
const opts={name:'aliased-spread',include:['tests/aliased-spread.test.mts'],setupFiles:['./old.setup.mts'], ...globalThis.options};
export default defineConfig(()=>({test:opts}));
