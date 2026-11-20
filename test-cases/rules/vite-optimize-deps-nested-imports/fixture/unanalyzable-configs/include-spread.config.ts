const more = [];

export default {
  optimizeDeps: {
    include: ["widgets > items", ...more],
  },
};
