import { defineConfig as config, defineProject as project } from 'vitest/config';
const workers = 1;
export default config({ test: { projects: [project({ test: { fileParallelism: false, maxWorkers: workers } })] } });
