export default {
  testDir: './tests/e2e',
  use: {
    // Statically readable, and intentionally different from the configured
    // `selectors.testIds` (data-pw). This value wins over the fallback.
    testIdAttribute: 'data-qa',
  },
};
