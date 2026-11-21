const label = 'widgets';
export default {
  test: {
    ...(label || { exclude: ['**/node_modules/**', '**/.git/**'] }),
    projects: [{ test: { name: 'orders' } }],
  },
};
