export default { test: {
  exclude: [],
  projects: [
    { extends: true, test: { name: 'widgets' } },
  ],
} };
