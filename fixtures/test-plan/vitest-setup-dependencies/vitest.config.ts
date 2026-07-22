import { defineConfig } from 'vitest/config'
import { dynamicSetup } from './config/setup-selector'
import { importedDynamicSetup } from './config/dynamic-wrapper'
import { importedSetupFiles } from './config/imported-setup-values'
import defaultImportedSetups from './config/default-imported-setups'
import defaultNamedImportedSetups from './config/default-named-setup-reexport'
import { sourceReexportedSetupFiles } from './config/source-setup-reexport'
import { importedReexportedSetupFiles } from './config/imported-setup-reexport'
import { barrelSetupFiles } from './config/setup-barrel'
import templateImportedSetup from './config/template-imported-setup'
import importedProject from './vitest.setup-imported'

const localDynamicSetup = () => importedDynamicSetup()
// This static reference cycle must stop after recording the config trigger.
const cyclicDynamicSetup = () => cyclicDynamicSetup()

export default defineConfig({
  test: {
    setupFiles: './setup/root.ts',
    globalSetup: './setup/global.mts',
    projects: [
      {
        test: {
          name: 'inherits',
          root: './inherits',
          include: ['**/*.test.ts'],
          // Vitest defaults inline projects to independent config; this one
          // deliberately exercises inherited root setup fields.
          extends: true,
          // Keep unsafe setup declarations project-scoped so resolved setup
          // changes can prove exact ownership without triggering this fallback.
          setupFiles: ['./setup/root.ts', dynamicSetup, './setup/missing.ts'],
        },
      },
      {
        test: {
          name: 'override',
          include: ['override/**/*.test.ts'],
          setupFiles: './setup/override.js',
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'imported-values',
          root: './imported-values',
          include: ['**/*.test.ts'],
          // Imported literal arrays may pass through normal runtime re-export
          // forms without becoming an unsafe dynamic setup declaration.
          setupFiles: [
            importedSetupFiles,
            defaultImportedSetups,
            defaultNamedImportedSetups,
            sourceReexportedSetupFiles,
            importedReexportedSetupFiles,
            barrelSetupFiles,
            templateImportedSetup,
          ],
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'dynamic-cycle',
          // Keep this recursion-guard fixture outside every real test owner.
          root: './cycle-owner',
          include: ['**/*.test.ts'],
          setupFiles: cyclicDynamicSetup,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'cleared',
          include: ['cleared/**/*.test.ts'],
          setupFiles: [],
          globalSetup: [],
        },
      },
      {
        test: {
          // This project intentionally has no matching test. Its known owner
          // must not widen an unsafe setup fallback to unrelated projects.
          name: 'empty-owner',
          root: './empty-owner',
          include: ['**/*.test.ts'],
          setupFiles: dynamicSetup,
        },
      },
      {
        test: {
          name: 'dynamic-closure',
          root: './closure-owner',
          include: ['**/*.test.ts'],
          // The transitive helper is outside this project root. Keep this
          // dynamic declaration so impact fallback must follow its closure.
          setupFiles: localDynamicSetup,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'alias-deleted',
          root: './alias-owner',
          include: ['**/*.test.ts'],
          // Keep the target absent: its deletion must still resolve through
          // the configured TypeScript alias during impact fallback.
          setupFiles: '@setup/missing',
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'base-url-index-deleted',
          root: './base-owner',
          include: ['**/*.test.ts'],
          // This extensionless baseUrl target exercises index-file parity.
          setupFiles: 'base-setup/missing',
          globalSetup: [],
        },
      },
      importedProject,
      './vitest.string-project.ts',
      './packages/foo/vitest.project.ts',
    ],
  },
})
