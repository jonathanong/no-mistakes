const path = require("node:path");
const base = require("../../../../vitest.config.js");

module.exports = {
  ...base,
  test: {
    ...base.test,
    setupFiles: [],
    globals: true,
    include: [path.join(__dirname, "*.test.cjs")],
  },
};
