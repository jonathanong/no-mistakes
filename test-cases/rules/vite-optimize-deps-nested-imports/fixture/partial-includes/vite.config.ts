const extra = ["widgets > ignored"];

export default {
  optimizeDeps: {
    include: ["widgets > items", ...extra],
  },
};
