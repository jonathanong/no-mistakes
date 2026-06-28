"use strict";

const rules = {
  "async-call-disposition": require("./rules/async-call-disposition"),
  "async-try-catch-return-await": require("./rules/async-try-catch-return-await"),
  "await-array-methods": require("./rules/await-array-methods"),
  "nextjs-static-fetch-method": require("./rules/nextjs-static-fetch-method"),
  "nextjs-static-fetch-url": require("./rules/nextjs-static-fetch-url"),
  "module-mock-boundary": require("./rules/module-mock-boundary"),
  "module-mock-preserve-exports": require("./rules/module-mock-preserve-exports"),
  "nextjs-metadata-exports-location": require("./rules/nextjs-metadata-exports-location"),
  "nextjs-no-manual-script-tags": require("./rules/nextjs-no-manual-script-tags"),
  "no-global-fetch-outside-helper": require("./rules/no-global-fetch-outside-helper"),
  "no-delete-property": require("./rules/no-delete-property"),
  "no-import-only-test-files": require("./rules/no-import-only-test-files"),
  "no-placeholder-never-type-exports": require("./rules/no-placeholder-never-type-exports"),
  "no-vitest-sequential": require("./rules/no-vitest-sequential"),
  "playwright-consistent-attribute": require("./rules/playwright-consistent-attribute"),
  "playwright-defaults": require("./rules/playwright-defaults"),
  "playwright-assertion-timeout-cap": require("./rules/playwright-assertion-timeout-cap"),
  "playwright-literals": require("./rules/playwright-literals"),
  "playwright-naming-convention": require("./rules/playwright-naming-convention"),
  "playwright-no-empty": require("./rules/playwright-no-empty"),
  "playwright-no-set-timeout": require("./rules/playwright-no-set-timeout"),
  "playwright-prefer-get-by-test-id": require("./rules/playwright-prefer-get-by-test-id"),
  "playwright-require-exported-component-attribute": require("./rules/playwright-require-exported-component-attribute"),
  "playwright-require-interactive-test-id": require("./rules/playwright-require-interactive-test-id"),
  "playwright-selector-priority": require("./rules/playwright-selector-priority"),
  "react-no-iife-in-jsx": require("./rules/react-no-iife-in-jsx"),
  "playwright-unique": require("./rules/playwright-unique"),
  "react-no-nullish-react-node": require("./rules/react-no-nullish-react-node"),
  "react-no-use-promise-resolve": require("./rules/react-no-use-promise-resolve"),
  "server-require-nullable-fetch-wrapper": require("./rules/server-require-nullable-fetch-wrapper"),
  "test-no-error-message-matching": require("./rules/test-no-error-message-matching"),
  "test-no-shared-state": require("./rules/test-no-shared-state"),
  "ts-no-export-renaming": require("./rules/ts-no-export-renaming"),
  "ts-no-function-aliases": require("./rules/ts-no-function-aliases"),
  "ts-preserve-null-option-defaults": require("./rules/ts-preserve-null-option-defaults"),
  "vitest-mock-test-file-naming": require("./rules/vitest-mock-test-file-naming"),
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
    "no-mistakes/no-delete-property": "error",
    "no-mistakes/no-placeholder-never-type-exports": "error",
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
    "no-mistakes/await-array-methods": "error",
    "no-mistakes/nextjs-metadata-exports-location": "error",
    "no-mistakes/nextjs-no-manual-script-tags": "error",
    "no-mistakes/no-import-only-test-files": "error",
    "no-mistakes/no-vitest-sequential": "error",
    "no-mistakes/playwright-assertion-timeout-cap": "error",
    "no-mistakes/playwright-consistent-attribute": ["error", { canonicalAttribute: "data-pw" }],
    "no-mistakes/playwright-naming-convention": "error",
    "no-mistakes/playwright-no-set-timeout": "error",
    "no-mistakes/playwright-prefer-get-by-test-id": "warn",
    "no-mistakes/playwright-require-interactive-test-id": "warn",
    "no-mistakes/playwright-selector-priority": "error",
    "no-mistakes/react-no-iife-in-jsx": "error",
    "no-mistakes/react-no-use-promise-resolve": "error",
    "no-mistakes/test-no-error-message-matching": "error",
    "no-mistakes/test-no-shared-state": "error",
    "no-mistakes/vitest-mock-test-file-naming": "error",
  },
};

module.exports = plugin;
