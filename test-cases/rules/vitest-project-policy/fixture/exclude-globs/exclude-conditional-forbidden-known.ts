const orders = true;
export default {
  test: {
    projects: [
      {
        test: {
          ...(orders ? { poolOptions: {} } : { poolOptions: {} }),
          name: 'items',
        },
      },
    ],
  },
};
