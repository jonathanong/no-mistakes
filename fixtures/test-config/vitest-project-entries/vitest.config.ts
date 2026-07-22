const spreadProjects = [
  './projects/z/vitest.config.ts',
  './projects/folder',
]

// This separate spread resolves to the same canonical config as the folder.
const overlappingProjects = ['./projects/folder/vite.config.ts']
const excludedProjects = ['!./projects/excluded/**']
// This static cycle must remain bounded while flattening project array spreads.
const recursiveProjects = [...recursiveProjects]
const functionProjectFactory = function () {
  return [{ name: 'function-expression' }]
}

export default {
  test: {
    projects: [
      ...spreadProjects,
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
      './projects/*',
      // Invalid globs have no static project candidates and are ignored.
      './projects/[',
      // This spread is deliberately last: negation still excludes the
      // invalid config before any project config is parsed.
      ...excludedProjects,
    ],
  },
}
