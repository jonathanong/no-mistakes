const label = 'widgets';
export default {
  test: {
    ...(label ? { exclude: ['**/node_modules/**', '**/.git/**'] } : widgetSettings),
    projects: [{ test: { name: 'items' } }],
  },
};
