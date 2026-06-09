// Standard suite. Intentionally a BROAD testDir (`./playwright`) with NO
// testIgnore for the credentialed subfolder — this is the collision the
// nested-config ownership fix must resolve. Do not "fix" by ignoring
// credentialed here; the tool must scope ownership to the deepest testDir.
export default {
  name: 'web',
  testDir: './playwright',
  use: {
    baseURL: 'http://localhost:3000',
    testIdAttribute: 'data-pw',
  },
  projects: [
    { name: 'chromium' },
  ],
};
