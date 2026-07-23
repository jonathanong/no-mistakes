import folderProjects from './project-folders'
import excludedProjectFolders from './excluded-project-folders'

export default {
  test: {
    // Configless folder projects must retain their own default include.
    include: ['root/**/*.spec.ts'],
    // Imported negations must filter configless roots introduced by another spread.
    projects: [
      './packages/direct/vitest.config.ts',
      ...excludedProjectFolders,
      ...folderProjects,
    ],
  },
}
