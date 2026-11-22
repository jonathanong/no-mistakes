import { configDefaults as defaults, defineConfig as config, defineProject } from 'vitest/config'
import { configDefaults as unrelatedDefaults } from 'another-runner'
import { type configDefaults as typeDefaults } from 'vitest/config'
import type { configDefaults as typeDeclarationDefaults } from 'vitest/config'
import { excludes, flag, overrides, replacementKey } from './dynamic'

export default config({
  exclude: [],
  test: {
    exclude: ['**/node_modules/**', 'tmp/**'],
    coverage: { exclude: [] },
    projects: [
      { test: {} },
      { test: { exclude: [] } },
      defineProject({ test: { name: 'complete', exclude: ['**/node_modules/**', '**/.git/**'] } }),
      { test: { exclude: [...defaults.exclude, 'tmp/**'] } },
      { test: { exclude: excludes } },
      { test: { exclude: getExcludes() } },
      { test: { exclude: flag ? [] : [] } },
      { test: { exclude: [flag] } },
      { test: { exclude: [...excludes] } },
      { test: { exclude: [...unrelatedDefaults.exclude] } },
      { test: { exclude: [...typeDefaults.exclude] } },
      { test: { exclude: [...typeDeclarationDefaults.exclude] } },
      { test: { exclude: [], ...overrides } },
      { test: { exclude: [], [replacementKey]: excludes } },
      { test: { ...overrides, exclude: [] } },
      { test: { exclude: [], 'exclude': ['**/node_modules/**', '**/.git/**'] } },
      { test: {
        // no-mistakes-disable-next-line vitest-default-excludes
        exclude: [],
      } },
      { test: { exclude: [] } }, // no-mistakes-disable-line vitest-default-excludes
      { test: { exclude: [`**/node_mod\u0075les/**`] } },
      { test: { exclude: [`**/node_modules/**`, `**/.git/**`] } },
      { test: { exclude: [`**/${flag}/**`] } },
      vitest.defineProject({ test: { exclude: [] } }),
      mergeConfig({ test: { exclude: [] } }),
      defineProject({ test: { exclude: [] } }, flag),
      { exclude: [], test: {} },
      'vitest.external.ts',
    ],
  },
})

import * as vitest from 'vitest/config'
import { mergeConfig } from 'vitest/config'
