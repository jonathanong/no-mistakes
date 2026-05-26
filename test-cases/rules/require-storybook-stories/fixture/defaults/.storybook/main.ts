const ignored = "stories";

export default {
  // "../ignored/**/*.stories.tsx"
  stories: [
    {
      directory: "../storybook",
      files: `**/*.stories.tsx`,
    },
  ],
};
