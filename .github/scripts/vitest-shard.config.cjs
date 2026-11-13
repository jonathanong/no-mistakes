const path = require("node:path");
const base = require("../../vitest.config.js");

module.exports = {
  ...base,
  root: path.resolve(__dirname, "../.."),
  test: {
    ...base.test,
    coverage: { ...base.test.coverage, thresholds: undefined },
  },
};
