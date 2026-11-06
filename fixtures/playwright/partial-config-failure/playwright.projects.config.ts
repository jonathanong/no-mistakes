export default {
  projects: [
    { name: 'good', testDir: './tests' },
    { name: 'broken', testDir: process.env.TEST_DIR },
  ],
};
