export default { test: {
  exclude: ['**/node_modules/**', '**/.git/**'],
  projects: [
    { test: { name: 'orders', exclude: ['**/.git/**'] } },
  ],
} };
