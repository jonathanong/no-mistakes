import * as cfg from 'vitest/config';
const projects = [cfg.defineProject({ test: { testTimeout: 60000 } })];
const expression = cfg.defineConfig(() => ({ test: { hookTimeout: 60000 } }));
const returned = cfg.defineConfig(function () { return { test: { testTimeout: 60000 } }; });
const base = cfg.defineConfig({ test: { projects: [, ...projects] } });
const more = cfg.defineProject({ test: { hookTimeout: 60000 } });
export default cfg.mergeConfig(base, { test: { projects: [more] } });
