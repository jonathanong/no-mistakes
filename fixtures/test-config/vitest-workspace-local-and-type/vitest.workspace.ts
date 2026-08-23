export type WorkspaceProjectName = string

const projects = [
  { test: { name: 'local-export-project', include: ['local/**/*.test.ts'] } },
]

export { projects as default }
