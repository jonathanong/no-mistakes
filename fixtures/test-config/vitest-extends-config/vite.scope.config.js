export default {
  test: {
    root: './scope-inherited',
    include: ['owned/**/*.spec.ts'],
    exclude: ['inherited-ignore/**'],
    setupFiles: './scope-setup.ts',
  },
}
