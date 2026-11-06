export default {
  test: {
    name: 'root',
    globalSetup: './root-setup.ts',
    isolate: true,
    projects: [{
      extends: true,
      test: {
        name: 'child',
        globalSetup: './child-setup.ts',
        projects: [],
      },
    }],
  },
};
