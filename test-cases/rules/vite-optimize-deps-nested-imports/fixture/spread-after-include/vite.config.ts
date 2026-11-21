const overrides = { include: [] };

export default {
  optimizeDeps: {
    include: ["widgets > items", "widgets > orders"],
    ...overrides,
  },
};
