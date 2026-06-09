// Helper that injects `testIdAttribute` into the Playwright config. no-mistakes
// does not parse this file; it only sees the call in playwright.config.ts, which
// is why the attribute is not statically readable.
export function createPlaywrightConfig(options: { testDir: string }) {
  return {
    ...options,
    use: {
      testIdAttribute: 'data-pw',
    },
  };
}
