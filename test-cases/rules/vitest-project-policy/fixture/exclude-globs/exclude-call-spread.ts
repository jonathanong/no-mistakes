export default {
  test: {
    ...loadWidgetSettings(),
    projects: [{ test: { name: 'orders' } }],
  },
};
