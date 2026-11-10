export default {
  test: {
    include: ["tests/**/*.test.mts"],
    setupFiles: [
      process.env.SETUP_A,
      process.env.SETUP_B,
    ],
  },
};
