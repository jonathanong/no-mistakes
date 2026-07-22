const spreadProjects = [
  './projects/z/vitest.config.ts',
  './projects/folder',
]

// This separate spread resolves to the same canonical config as the folder.
const overlappingProjects = ['./projects/folder/vite.config.ts']
const excludedProjects = ['!./projects/excluded/**']

export default {
  test: {
    projects: [
      ...spreadProjects,
      ...overlappingProjects,
      { name: 'inline-z' },
      // Inline literal spreads are valid static project entries too.
      ...[{ name: 'inline-direct-spread' }],
      './projects/a/vitest.config.ts',
      { name: 'inline-a' },
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
