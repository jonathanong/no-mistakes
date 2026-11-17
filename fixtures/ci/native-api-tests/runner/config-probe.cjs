const config = require("./vitest.config.cjs");

process.stdout.write(
  JSON.stringify({
    fileParallelism: config.test.fileParallelism,
    testTimeout: config.test.testTimeout,
  }),
);
