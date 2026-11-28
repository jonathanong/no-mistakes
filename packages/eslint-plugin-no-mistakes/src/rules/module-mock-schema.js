"use strict";

const { UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");

const stringList = { type: "array", items: { type: "string" } };

const integrationExports = {
  type: "object",
  properties: {
    extensions: stringList,
    markerRegex: { type: "string" },
    reexportExtensions: stringList,
    sourcePathTemplates: stringList,
    specifierPrefix: { type: "string" },
    specifiers: stringList,
  },
};

function baseline(types) {
  return {
    type: "array",
    items: {
      type: "array",
      minItems: types.length,
      maxItems: types.length,
      items: types.map((type) => ({ type })),
    },
  };
}

const shared = {
  excludePathPatterns: stringList,
  includePathPatterns: stringList,
  internalSpecifiers: stringList,
  unanalyzable: UNANALYZABLE_SCHEMA,
};

const boundaryOptions = {
  type: "object",
  properties: {
    ...shared,
    baseline: baseline(["string", "string", "number"]),
    integrationExports,
    requireLiteralSpecifiers: { type: "boolean" },
  },
};

const preserveOptions = {
  type: "object",
  properties: {
    ...shared,
    baseline: baseline(["string", "string"]),
  },
};

module.exports = { boundaryOptions, preserveOptions };
