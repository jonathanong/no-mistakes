import { defineConfig } from 'vitest/config';
import { projects } from './vitest.projects.mts';

export default defineConfig({ test: { projects } });
