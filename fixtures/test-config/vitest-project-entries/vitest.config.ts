import importedProjects from './projects/imported-projects'
import importedExclusions from './projects/imported-exclusions'
import defaultFunctionExclusions from './projects/default-function-exclusions'
import defaultFunctionIdentifierExclusions from './projects/default-function-identifier-exclusions'
import defaultImportedExclusions from './projects/default-imported-exclusions'
import { namedFunctionExclusions } from './projects/named-function-exclusions'

const spreadProjects = [
  './projects/z/vitest.config.ts',
  './projects/folder',
]

// This separate spread resolves to the same canonical config as the folder.
const overlappingProjects = ['./projects/folder/vite.config.ts']
const excludedProjects = ['!./projects/excluded/**']
const localExclusions = () => {
  const exclusions = ['!./projects/negated-local.config.ts']
  return exclusions
}
// This static cycle must remain bounded while flattening project array spreads.
const recursiveProjects = [...recursiveProjects]
const functionProjectFactory = function () {
  return [{ name: 'function-expression' }]
}

export default {
  test: {
    projects: [
      ...spreadProjects,
      ...importedProjects,
      ...overlappingProjects,
      ...recursiveProjects,
      { name: 'inline-z' },
      // Inline literal spreads are valid static project entries too.
      ...[{ name: 'inline-direct-spread' }],
      './projects/a/vitest.config.ts',
      { name: 'inline-a' },
      ...functionProjectFactory(),
      './projects/self/vitest.config.ts',
      './projects/direct.config.ts',
      // This positive config is negated by the imported project array below.
      // It has setup state so a skipped config must not leak that ownership.
      './projects/negated-outer.config.ts',
      './projects/negated-local.config.ts',
      './projects/negated-imported.config.ts',
      './projects/negated-default-function.config.ts',
      './projects/negated-default-function-identifier.config.ts',
      './projects/negated-default-imported.config.ts',
      './projects/negated-named-function.config.ts',
      // Static zero-argument helpers participate in global exclusions too.
      ...localExclusions(),
      ...importedExclusions(),
      ...defaultFunctionExclusions(),
      ...defaultFunctionIdentifierExclusions(),
      ...defaultImportedExclusions,
      ...namedFunctionExclusions(),
      // This outer negation must also exclude config strings from imported arrays.
      '!./projects/imported-excluded/vitest.config.ts',
      './projects/*',
      // Invalid globs have no static project candidates and are ignored.
      './projects/[',
      // This spread is deliberately last: negation still excludes the
      // invalid config before any project config is parsed.
      ...excludedProjects,
    ],
  },
}
