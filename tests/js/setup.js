const { join } = require("node:path");

process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH ||= join(
  __dirname,
  "..",
  "..",
  "fixtures",
  "napi",
  "test-addon.js",
);
