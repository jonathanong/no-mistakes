"use strict";

const { unwrapExpression } = require("./async-ast");
const { compileTargets, matchingTargets } = require("./async-patterns");
const { isDefinitelyUndefinedValue } = require("./imported-call-undefined");
const { propertyName, literalString } = require("./module-mock-helpers");

const patternList = { type: "array", items: { type: "string" } };

const importedCallOptionsSchema = [
  {
    type: "object",
    properties: {
      targets: {
        type: "array",
        items: {
          type: "object",
          properties: {
            sourceSpecifierPatterns: patternList,
            calleeNamePatterns: patternList,
            globalNamePatterns: patternList,
            receiverPatterns: patternList,
            optionsPosition: { type: "integer", minimum: 1 },
            requiredProperties: { type: "array", items: { type: "string" }, minItems: 1 },
            propertyMatch: { type: "string", enum: ["any", "all"] },
            message: { type: "string", minLength: 1 },
          },
          required: ["optionsPosition", "requiredProperties"],
          additionalProperties: false,
        },
      },
      message: { type: "string", minLength: 1 },
      checkedPathPatterns: patternList,
      allowedPathPatterns: patternList,
    },
    additionalProperties: false,
  },
];

function normalizeRequiredProperties(value) {
  return Array.isArray(value)
    ? [...new Set(value.filter((name) => typeof name === "string" && name))]
    : [];
}

function compileImportedCallTargets(options) {
  const compiled = [];
  for (const target of options.targets || []) {
    const [matched] = compileTargets({ targets: [target] }, "targets");
    const optionsPosition = target.optionsPosition;
    const requiredProperties = normalizeRequiredProperties(target.requiredProperties);
    if (
      !matched ||
      !Number.isInteger(optionsPosition) ||
      optionsPosition < 1 ||
      requiredProperties.length === 0
    ) {
      continue;
    }
    const next = {
      ...matched,
      optionsPosition,
      requiredProperties,
      propertyMatch: target.propertyMatch === "all" ? "all" : "any",
    };
    // Per-target text stays off the object unless set, so older compiled shapes stay stable.
    if (typeof target.message === "string" && target.message) next.message = target.message;
    compiled.push(next);
  }
  return compiled;
}

function staticPropertyName(property) {
  if (property.type !== "Property") return null;
  if (property.computed) return literalString(unwrapExpression(property.key));
  return propertyName(property.key);
}

// Later static keys replace earlier ones. A spread neither adds nor erases a key.
function visiblePropertyValues(node) {
  const values = new Map();
  if (node?.type !== "ObjectExpression") return values;
  for (const property of node.properties) {
    const name = staticPropertyName(property);
    if (name) values.set(name, property.value);
  }
  return values;
}

function visiblePropertyNames(node) {
  return new Set(visiblePropertyValues(node).keys());
}

function presentPropertyNames(node, sourceCode) {
  const names = new Set();
  for (const [name, value] of visiblePropertyValues(node)) {
    if (!isDefinitelyUndefinedValue(value, sourceCode)) names.add(name);
  }
  return names;
}

function hasRequiredOptions(argument, target, sourceCode) {
  const object = unwrapExpression(argument);
  if (object?.type !== "ObjectExpression") return false;
  const names = presentPropertyNames(object, sourceCode);
  if (target.propertyMatch === "all") {
    return target.requiredProperties.every((name) => names.has(name));
  }
  return target.requiredProperties.some((name) => names.has(name));
}

module.exports = {
  compileImportedCallTargets,
  hasRequiredOptions,
  importedCallOptionsSchema,
  isDefinitelyUndefinedValue,
  matchingTargets,
  normalizeRequiredProperties,
  staticPropertyName,
  visiblePropertyNames,
};
