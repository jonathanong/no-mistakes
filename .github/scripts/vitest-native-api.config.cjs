const base = require("../../vitest.config.js");
const { readShardTestPattern } = require("./native-api-shards.cjs");

module.exports = {
  ...base,
  test: {
    ...base.test,
    testNamePattern: readShardTestPattern(),
  },
};
