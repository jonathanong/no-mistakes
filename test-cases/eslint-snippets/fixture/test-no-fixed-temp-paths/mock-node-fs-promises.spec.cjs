const { readFile } = require("node:fs/promises");
const { jest: runner } = require("@jest/globals");

readFile("/tmp/mock-file");
runner["mock"]("node:fs/promises");
