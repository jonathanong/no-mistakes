export default {
  testDir: './tests/e2e',
  use: {
    // Statically readable, but overridden by tests.playwright.testIdAttribute.
    testIdAttribute: 'data-qa',
  },
};
