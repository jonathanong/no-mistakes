const shared = { optimizeDeps: { include: [] } };

export default {
  optimizeDeps: {
    include: ["widgets > items", "widgets > orders"],
  },
  ...shared,
};
