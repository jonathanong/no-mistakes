const base = { retry: 1 };
export default {
  test: {
    ...(base && { exclude: ['**/node_modules/**'] }),
    projects: [],
  },
};
