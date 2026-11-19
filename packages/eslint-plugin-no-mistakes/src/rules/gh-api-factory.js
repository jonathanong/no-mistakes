"use strict";
const { unwrapSequence } = require("./async-ast");
const { memberPropertyName } = require("./async-target-bindings");
const { wrapperName } = require("./gh-api-wrappers");

const factorySchema = {
  type: "array",
  maxItems: 64,
  items: {
    type: "object",
    properties: {
      factory: { type: "string", minLength: 1 },
      method: { type: "string", minLength: 1 },
      argumentPosition: { type: "integer", minimum: 1, maximum: 32 },
      includesApi: { type: "boolean" },
    },
    required: ["factory", "method", "argumentPosition"],
    additionalProperties: false,
  },
};

function createFactoryResolver(context, entries) {
  const roots = new Set(entries.map((entry) => entry.factory.split(".")[0]));
  return (node) => {
    const callee = unwrapSequence(node.callee);
    if (callee?.type !== "MemberExpression") return null;
    const receiver = unwrapSequence(callee.object);
    if (receiver?.type !== "CallExpression") return null;
    const method = memberPropertyName(callee);
    const factory = method && wrapperName(receiver.callee, context, roots);
    const entry = entries.find((item) => item.factory === factory && item.method === method);
    return entry ? { wrapper: entry, arguments: node.arguments } : null;
  };
}
module.exports = { createFactoryResolver, factorySchema };
