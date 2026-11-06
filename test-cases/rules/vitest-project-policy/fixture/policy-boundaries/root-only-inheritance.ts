export default {
  test: {
    name: 'root',
    globalSetup: './root-setup.ts',
    isolate: true,
    projects: [
      { extends: true, test: {} },
      { extends: true, test: { isolate: true } },
    ],
  },
};
