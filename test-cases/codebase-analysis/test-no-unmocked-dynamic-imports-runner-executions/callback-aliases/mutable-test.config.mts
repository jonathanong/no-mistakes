import { defineConfig } from 'vitest/config';
let opts={name:'mutable-test',include:['tests/mutable-test.test.mts'],setupFiles:['./old.setup.mts']};
opts={};
export default defineConfig(()=>({test:opts}));
