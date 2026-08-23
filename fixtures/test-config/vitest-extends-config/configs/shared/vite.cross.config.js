export default {
  test: {
    root: './inherited-root',
    include: ['inherited/**/*.spec.ts'],
    exclude: ['inherited-ignore/**'],
    setupFiles: './setup.ts',
  },
}
