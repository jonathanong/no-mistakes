"use strict";

const rules = {
  "nextjs-static-fetch-method": require("./rules/nextjs-static-fetch-method"),
  "nextjs-static-fetch-url": require("./rules/nextjs-static-fetch-url"),
  "playwright-consistent-attribute": require("./rules/playwright-consistent-attribute"),
  "playwright-defaults": require("./rules/playwright-defaults"),
  "playwright-literals": require("./rules/playwright-literals"),
  "playwright-naming-convention": require("./rules/playwright-naming-convention"),
  "playwright-no-empty": require("./rules/playwright-no-empty"),
  "playwright-prefer-get-by-test-id": require("./rules/playwright-prefer-get-by-test-id"),
  "playwright-require-interactive-test-id": require("./rules/playwright-require-interactive-test-id"),
  "playwright-unique": require("./rules/playwright-unique"),
  "react-no-nullish-react-node": require("./rules/react-no-nullish-react-node"),
  "ts-no-export-renaming": require("./rules/ts-no-export-renaming"),
  "ts-no-function-aliases": require("./rules/ts-no-function-aliases"),
};

const plugin = {
  meta: {
    name: "eslint-plugin-no-mistakes",
    version: require("../package.json").version,
  },
  rules,
  configs: {},
};

plugin.configs.recommended = {
  plugins: {
    "no-mistakes": plugin,
  },
  rules: {
    "no-mistakes/nextjs-static-fetch-method": "error",
    "no-mistakes/nextjs-static-fetch-url": "error",
    "no-mistakes/playwright-defaults": "error",
    "no-mistakes/playwright-literals": "error",
    "no-mistakes/playwright-no-empty": "error",
    "no-mistakes/playwright-unique": "error",
    "no-mistakes/react-no-nullish-react-node": "error",
    "no-mistakes/ts-no-export-renaming": "error",
    "no-mistakes/ts-no-function-aliases": "error",
  },
};

plugin.configs.strict = {
  plugins: {
    "no-mistakes": plugin,
  },
  rules: {
    ...plugin.configs.recommended.rules,
    "no-mistakes/playwright-consistent-attribute": ["error", { canonicalAttribute: "data-pw" }],
    "no-mistakes/playwright-naming-convention": "error",
    "no-mistakes/playwright-prefer-get-by-test-id": "warn",
    "no-mistakes/playwright-require-interactive-test-id": "warn",
  },
};

module.exports = plugin;
