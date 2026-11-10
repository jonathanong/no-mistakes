const baseProject = {
  test: {
    setupFiles: (["./setup.mts"] as const),
  },
} satisfies { test: { setupFiles: readonly string[] } };

export default {
  test: {
    include: ["tests/**/*.test.mts"],
    setupFiles: [...(baseProject.test.setupFiles)],
  },
};
