// An exact folder project string must not import-resolve this index module.
export default {
  test: {
    name: 'index-module-is-not-a-project-config',
    setupFiles: './setup.ts',
  },
}
