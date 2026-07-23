import folderProjects from './project-folders'

export default {
  test: {
    // Configless folder projects must retain their own default include.
    include: ['root/**/*.spec.ts'],
    // Root negation must filter configless roots introduced by this imported spread.
    projects: ['./packages/direct/vitest.config.ts', '!packages/skip', ...folderProjects],
  },
}
