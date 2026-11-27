const { readFileSync } = require("node:fs");
const base = require("../../vitest.config.js");

const { pattern } = JSON.parse(readFileSync("target/native-api-shard.json", "utf8"));
if (typeof pattern !== "string" || pattern.length === 0)
  throw new Error("native API shard plan requires a test filter");

module.exports = {
  ...base,
  test: {
    ...base.test,
    testNamePattern: new RegExp(pattern),
  },
};
