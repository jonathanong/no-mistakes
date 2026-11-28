"use strict";

const { UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");

const stringList = { type: "array", items: { type: "string" } };

const integrationExports = {
  type: "object",
  additionalProperties: false,
  properties: {
    extensions: stringList,
    markerRegex: { type: "string" },
    reexportExtensions: stringList,
    sourcePathTemplates: stringList,
    specifierPrefix: { type: "string" },
    specifiers: stringList,
  },
};

function baseline(maxItems) {
  return {
    type: "array",
    items: {
      type: "array",
      minItems: maxItems,
      maxItems,
      items: { type: ["string", "number"] },
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
  additionalProperties: false,
  properties: {
    ...shared,
    baseline: baseline(3),
    integrationExports,
    requireLiteralSpecifiers: { type: "boolean" },
  },
};

const preserveOptions = {
  type: "object",
  additionalProperties: false,
  properties: {
    ...shared,
    baseline: baseline(2),
  },
};

module.exports = { boundaryOptions, preserveOptions };
