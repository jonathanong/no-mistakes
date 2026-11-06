import shared, * as helpers from './helper';
import { defineConfig, defineProject } from 'vitest/config';
const projects = [
  defineProject({ test: { ...shared, name: 'first', isolate: true, nullable: null, environment: 'node' } }),
  { test: { ...helpers.pair, name: 'second', isolate: true, nullable: null, environment: 'node' } },
] as const;
export default defineConfig({ test: { projects: [...projects] } });
