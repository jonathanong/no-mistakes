export default {
  test: { include: ["tests/**/*.test.mts"], setupFiles: [process.env.TEST_SETUP] },
};
