import { defineConfig } from 'vitest/config'
import { dynamicSetup } from './config/setup-selector'
import { importedDynamicSetup } from './config/dynamic-wrapper'
import { importedCommonjsDynamicSetup } from './config/dynamic-commonjs-wrapper'
import { importedSetupFiles } from './config/imported-setup-values'
import defaultImportedSetups from './config/default-imported-setups'
import defaultNamedImportedSetups from './config/default-named-setup-reexport'
import { sourceReexportedSetupFiles } from './config/source-setup-reexport'
import { importedReexportedSetupFiles } from './config/imported-setup-reexport'
import { barrelSetupFiles } from './config/setup-barrel'
import templateImportedSetup from './config/template-imported-setup'
import importedProject from './vitest.setup-imported'
import * as namespaceSetups from './config/namespace-setups'
import commonjsDefaultSetups, {
  namedSetups as commonjsNamedSetups,
  moduleNamedSetups as commonjsModuleNamedSetups,
} from './config/commonjs-setups.cjs'
import reexportedDefaultSetups from './config/default-imported-through-local'
import { declarationOnlySetups } from './config/declaration-only-setups'
import { missingBarrelSetups } from './config/missing-setup-barrel'
import { useAlternateSetup } from './config/branch-selector'
import { namedMemberConfig } from './config/named-member-setups'
import { sourcedNamedMemberConfig } from './config/named-member-source-reexport'
import { importedNamedMemberConfig } from './config/named-member-imported-reexport'
import { starNamedMemberConfig } from './config/named-member-star-barrel'
import { commonjsNamedMemberConfig } from './config/named-member-commonjs.cjs'
import { cycleMemberConfig } from './config/named-member-cycle-a'

const localSetups = { files: ['./setup/local-member.ts'] }
const requiredSetups = require('./config/commonjs-require-setups.cjs').setupFiles
// Destructured CJS aliases remain static setup bindings, not dynamic values.
const { destructuredSetups: aliasedDestructuredSetups } = require('./config/module-exports-object-setups.cjs')
const objectMemberSetups = require('./config/module-exports-object-setups.cjs').objectMemberSetups
const replacedObjectSetups = require('./config/commonjs-replacement-setups.cjs').replacedObjectSetups
const aliasBarrierSetups = require('./config/commonjs-replacement-setups.cjs').aliasBarrierSetups
const moduleOverrideSetups = require('./config/commonjs-replacement-setups.cjs').moduleOverrideSetups
const replacedNonobjectSetups = require('./config/commonjs-nonobject-replacement-setups.cjs').replacedNonobjectSetups
const { projects: cjsNamedMemberProjects } = require('./config/cjs-project-named-member.cjs')
const { projects: cjsNamedObjectProjects } = require('./config/cjs-project-named-object.cjs')
const { projects: cjsNamedReplacementProjects } = require('./config/cjs-project-named-replacement.cjs')

const localDynamicSetup = () => importedDynamicSetup()
const localCommonjsDynamicSetup = () => importedCommonjsDynamicSetup()
// This static reference cycle must stop after recording the config trigger.
const cyclicDynamicSetup = () => cyclicDynamicSetup()

export default defineConfig({
  test: {
    // Configless project folders must not inherit this aggregate-only glob.
    include: ['root/**/*.spec.ts'],
    setupFiles: './setup/root.ts',
    globalSetup: './setup/global.mts',
    projects: [
      {
        test: {
          name: 'local-member',
          include: ['local-member/**/*.test.ts'],
          setupFiles: localSetups.files,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'namespace-member',
          include: ['namespace-member/**/*.test.ts'],
          setupFiles: namespaceSetups.files,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'named-import-member',
          root: './named-member-owner',
          include: ['**/*.test.ts'],
          // The setup is shared outside this owner. Named imported object
          // members must retain an exact setup edge and helper provenance.
          setupFiles: [
            namedMemberConfig.files,
            sourcedNamedMemberConfig.files,
            importedNamedMemberConfig.files,
            starNamedMemberConfig.files,
            commonjsNamedMemberConfig.files,
          ],
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'named-import-member-cycle',
          root: './named-member-cycle-owner',
          include: ['**/*.test.ts'],
          // Re-export cycles remain a bounded dynamic fallback.
          setupFiles: cycleMemberConfig.files,
          globalSetup: [],
        },
      },
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
          name: 'commonjs-values',
          root: './commonjs-values',
          include: ['**/*.test.ts'],
          // CJS helpers can provide literal setup arrays without execution.
          setupFiles: [
            commonjsDefaultSetups,
            commonjsNamedSetups,
            commonjsModuleNamedSetups,
          ],
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'imported-default-local-reexport',
          root: './imported-default-local-reexport',
          include: ['**/*.test.ts'],
          setupFiles: reexportedDefaultSetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'commonjs-require',
          root: './commonjs-require',
          include: ['**/*.test.ts'],
          // Static CommonJS config bindings are equivalent to literal imports.
          setupFiles: requiredSetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'destructured-commonjs-require',
          root: './destructured-commonjs-require',
          include: ['**/*.test.ts'],
          setupFiles: aliasedDestructuredSetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'module-exports-object',
          root: './module-exports-object',
          include: ['**/*.test.ts'],
          setupFiles: objectMemberSetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'arbitrary-project-match',
          root: './arbitrary-project-match',
          include: ['**/*.fixture'],
          setupFiles: './setup/arbitrary.ts',
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'runtime-loaders',
          root: './runtime-owner',
          include: ['**/*.test.ts'],
          setupFiles: [
            './setup/runtime-loaders.ts',
            './setup/deleted-runtime-loader.ts',
          ],
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'declaration-only',
          root: './declaration-only',
          include: ['**/*.test.ts'],
          // A declaration file is not a runtime setup helper or fallback trigger.
          setupFiles: declarationOnlySetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'missing-barrel',
          root: './missing-barrel',
          include: ['**/*.test.ts'],
          // The barrel remains parseable while its runtime leaf is absent.
          setupFiles: missingBarrelSetups,
          globalSetup: [],
        },
      },
      {
        test: {
          name: 'conditional-setup',
          root: './conditional-owner',
          include: ['**/*.test.ts'],
          // Both literal branches are statically known. The selector lives
          // outside the owner so its provenance must remain explicit.
          setupFiles: useAlternateSetup
            ? '../setup/conditional-a.ts'
            : '../setup/conditional-b.ts',
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
          name: 'commonjs-replacement',
          root: './commonjs-replacement-owner',
          include: ['**/*.test.ts'],
          // Final module replacement detaches `exports`, while a later
          // `module.exports.member` assignment remains an exact override.
          setupFiles: [
            replacedObjectSetups,
            replacedNonobjectSetups,
            aliasBarrierSetups,
            moduleOverrideSetups,
          ],
          globalSetup: [],
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
          name: 'dynamic-commonjs-closure',
          root: './commonjs-closure-owner',
          include: ['**/*.test.ts'],
          // This external dynamic helper reaches its value through require.
          setupFiles: localCommonjsDynamicSetup,
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
      // Literal CommonJS project arrays work both as entries and spreads.
      require('./config/cjs-project-direct-element.cjs'),
      ...require('./config/cjs-project-direct-spread.cjs'),
      ...cjsNamedMemberProjects,
      ...cjsNamedObjectProjects,
      ...cjsNamedReplacementProjects,
      // Cycles and computed CommonJS exports remain deliberately empty.
      ...require('./config/cjs-project-cycle-a.cjs'),
      ...require('./config/cjs-project-computed.cjs'),
      importedProject,
      './vitest.string-project.ts',
      './vitest.standalone-imported-project.ts',
      './packages/foo/vitest.project.ts',
      './configless-project',
    ],
  },
})
