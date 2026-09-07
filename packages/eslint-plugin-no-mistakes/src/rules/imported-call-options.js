"use strict";

const { unwrapExpression } = require("./async-ast");
const { compileTargets, matchingTargets } = require("./async-patterns");
const { propertyName } = require("./module-mock-helpers");

const importedCallOptionsSchema = [
  {
    type: "object",
    properties: {
      targets: {
        type: "array",
        items: {
          type: "object",
          properties: {
            sourceSpecifierPatterns: { type: "array", items: { type: "string" } },
            calleeNamePatterns: { type: "array", items: { type: "string" } },
            optionsPosition: { type: "integer", minimum: 1 },
            requiredProperties: { type: "array", items: { type: "string" }, minItems: 1 },
            propertyMatch: { type: "string", enum: ["any", "all"] },
          },
          required: [
            "sourceSpecifierPatterns",
            "calleeNamePatterns",
            "optionsPosition",
            "requiredProperties",
          ],
          additionalProperties: false,
        },
      },
    },
    additionalProperties: false,
  },
];

function compileImportedCallTargets(options) {
  const compiled = [];
  for (const target of options.targets || []) {
    const [matched] = compileTargets({ targets: [target] }, "targets");
    const optionsPosition = target.optionsPosition;
    const requiredProperties = Array.isArray(target.requiredProperties)
      ? [...new Set(target.requiredProperties.filter((name) => typeof name === "string" && name))]
      : [];
    if (
      !matched ||
      !Number.isInteger(optionsPosition) ||
      optionsPosition < 1 ||
      requiredProperties.length === 0
    ) {
      continue;
    }
    compiled.push({
      ...matched,
      optionsPosition,
      requiredProperties,
      propertyMatch: target.propertyMatch === "all" ? "all" : "any",
    });
  }
  return compiled;
}

function staticPropertyName(property) {
  if (property.type !== "Property") return null;
  if (property.computed) {
    return property.key.type === "Literal" ? String(property.key.value) : null;
  }
  return propertyName(property.key);
}

function visiblePropertyNames(node) {
  const names = new Set();
  if (node?.type !== "ObjectExpression") return names;
  for (const property of node.properties) {
    const name = staticPropertyName(property);
    if (name) names.add(name);
  }
  return names;
}

function hasRequiredOptions(argument, target) {
  const object = unwrapExpression(argument);
  if (object?.type !== "ObjectExpression") return false;
  const names = visiblePropertyNames(object);
  if (target.propertyMatch === "all") {
    return target.requiredProperties.every((name) => names.has(name));
  }
  return target.requiredProperties.some((name) => names.has(name));
}

module.exports = {
  compileImportedCallTargets,
  hasRequiredOptions,
  importedCallOptionsSchema,
  matchingTargets,
  staticPropertyName,
  visiblePropertyNames,
};
