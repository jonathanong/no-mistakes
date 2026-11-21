const label = 'widgets';
export default {
  test: {
    ...(label ? { name: 'widgets' } : {}),
    projects: [
      {
        test: {
          name: 'items',
          ...(label ? { isolate: true } : {}),
        },
      },
    ],
  },
};
