import { defineConfig } from '@playwright/test'
import {
  factoryPlaywrightProjects,
  importedPlaywrightProjects,
  importedSpreadProjects,
  selfImportedSpreadProjects,
  wrappedPlaywrightProjects,
} from './playwright.projects-helper'

export default defineConfig(({
  name: 'root',
  testDir: './root',
  testIgnore: '**/root-ignore.ts',
  projects: [
    ...importedPlaywrightProjects,
    ...importedSpreadProjects,
    ...selfImportedSpreadProjects,
    ...factoryPlaywrightProjects(),
    ...wrappedPlaywrightProjects,
    {
      name: `absolute`,
      testDir: '/tmp/no-mistakes-absolute-tests',
      testMatch: [`**/*.spec.ts`],
      testIgnore: '**/skip.ts',
    },
    {
      name: 'inherits',
      testMatch: ['**/*.test.ts'],
    },
    {
      ['name']: 'computed',
      method() {},
    },
    1,
  ],
}))
