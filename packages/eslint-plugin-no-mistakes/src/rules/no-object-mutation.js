"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { scanMutations } = require("./no-object-mutation-scan");

const MESSAGE =
  "This mutates a shared object, so its contents can't be determined statically. Declare a literal once instead of writing into this one.";

function normalizeOptions(option) {
  const value = option || {};
  return {
    module: value.module !== false,
    imported: value.imported !== false,
    parameters: value.parameters !== false,
    calls: value.calls !== false,
    allowThis: value.allowThis !== false,
    allow: Array.isArray(value.allow) ? value.allow : [],
  };
}

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "disallow mutations that make shared objects statically unknowable",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          module: { type: "boolean" },
          imported: { type: "boolean" },
          parameters: { type: "boolean" },
          calls: { type: "boolean" },
          allowThis: { type: "boolean" },
          allow: {
            type: "array",
            items: { type: "string", minLength: 1 },
            uniqueItems: true,
          },
        },
        additionalProperties: false,
      },
    ],
    messages: { mutation: MESSAGE },
  },
  create(context) {
    const options = normalizeOptions(context.options[0]);
    const suppressed = ruleSuppression(context, "no-object-mutation");
    return {
      Program(program) {
        for (const node of scanMutations(program, { suppressed }, options)) {
          context.report({ node, messageId: "mutation" });
        }
      },
    };
  },
};
