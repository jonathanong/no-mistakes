import { defineProject } from 'vitest/config';

export default defineProject(() => {
  return {
    test: {
      include: ['tests/comments.test.mts'],
      // setupFiles: ['./comment.setup.mts'],
      description: "setupFiles: ['./comment.setup.mts']",
    },
  };
});
