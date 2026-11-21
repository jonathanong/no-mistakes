const label = 'widgets';
export default {
  test: {
    ...(label
      ? { exclude: ['**/node_modules/**', '**/.git/**'] }
      : { exclude: ['**/node_modules/**', '**/.git/**'] }),
    ...(label
      ? { exclude: ['**/node_modules/**', '**/.git/**', '**/widgets/**'] }
      : { exclude: ['**/node_modules/**', '**/.git/**', '**/orders/**'] }),
    projects: [{ test: { name: 'items', isolate: true } }],
  },
};
