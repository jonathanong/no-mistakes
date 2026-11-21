const label = 'widgets';
export default {
  test: {
    ...(label
      ? { exclude: ['**/node_modules/**'] }
      : { exclude: ['**/node_modules/**', '**/.git/**'] }),
    projects: [{ test: { name: 'items' } }],
  },
};
