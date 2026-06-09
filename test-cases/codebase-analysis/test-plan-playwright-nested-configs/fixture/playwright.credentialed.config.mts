// Credentialed suite. Its testDir (`./playwright/credentialed`) is nested
// inside the standard config's testDir and shares the same project name
// (`chromium`). Specs here belong to THIS config only.
export default {
  name: 'credentialed',
  testDir: './playwright/credentialed',
  use: {
    baseURL: 'http://localhost:3000',
    testIdAttribute: 'data-pw',
  },
  projects: [
    { name: 'chromium' },
  ],
};
