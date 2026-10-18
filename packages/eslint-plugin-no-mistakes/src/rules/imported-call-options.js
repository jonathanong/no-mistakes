"use strict";

const { unwrapExpression } = require("./async-ast");
const { compileTargets, matchingTargets } = require("./async-patterns");
const { isDefinitelyUndefinedValue } = require("./imported-call-undefined");
const { propertyName, literalString } = require("./module-mock-helpers");

const pathPatternList = { type: "array", items: { type: "string" } };
// A leading `/` is a wrapped regex, not a glob. The body cannot contain an
// unescaped `/`, so `/^@acme/http$/` and `/abs/**` fail here instead of
// matching a package path or an absolute glob. Escape slashes
// (`/^@acme\/http$/`) or write a glob that does not start with `/`.
// An unclosed class such as `/[/` must also fail before the rule can drop
// the target and match nothing.
const selectorPattern = {
  type: "string",
  minLength: 1,
  pattern: "^(?:[^/].*|/(?:[^/\\\\\\[\\]]|\\\\.|\\[[^\\]\\\\]*(?:\\\\.[^\\]\\\\]*)*\\])*/)$",
};
const selectorPatterns = { type: "array", minItems: 1, items: selectorPattern };
const targetOptions = {
  optionsPosition: { type: "integer", minimum: 1 },
  requiredProperties: { type: "array", items: { type: "string" }, minItems: 1 },
  propertyMatch: { type: "string", enum: ["any", "all"] },
  message: { type: "string", minLength: 1 },
};

function targetKind(required, properties) {
  return {
    type: "object",
    additionalProperties: false,
    required: [...required, "optionsPosition", "requiredProperties"],
    properties: { ...properties, ...targetOptions },
  };
}

const importedCallOptionsSchema = [
  {
    type: "object",
    additionalProperties: false,
    properties: {
      targets: {
        type: "array",
        items: {
          oneOf: [
            targetKind(["sourceSpecifierPatterns", "calleeNamePatterns"], {
              sourceSpecifierPatterns: selectorPatterns,
              calleeNamePatterns: selectorPatterns,
            }),
            targetKind(["globalNamePatterns"], { globalNamePatterns: selectorPatterns }),
            targetKind(["receiverPatterns", "calleeNamePatterns"], {
              receiverPatterns: selectorPatterns,
              calleeNamePatterns: selectorPatterns,
            }),
          ],
        },
      },
      message: { type: "string", minLength: 1 },
      checkedPathPatterns: pathPatternList,
      allowedPathPatterns: pathPatternList,
    },
  },
];

function normalizeRequiredProperties(value) {
  return Array.isArray(value)
    ? [...new Set(value.filter((name) => typeof name === "string" && name))]
    : [];
}

// Slash-wrapped patterns are regexes. `/(/` and `/*/` pass the schema, and
// `compileTargets` would drop them so the rule loads and checks nothing.
function assertCompilableSelectorPatterns(patterns) {
  if (!Array.isArray(patterns)) return;
  for (const pattern of patterns) {
    if (typeof pattern !== "string" || pattern.length < 3) continue;
    if (!pattern.startsWith("/") || !pattern.endsWith("/")) continue;
    try {
      RegExp(pattern.slice(1, -1));
    } catch (error) {
      throw new Error(`Invalid selector pattern ${pattern}: ${error.message}`);
    }
  }
}

function compileImportedCallTargets(options) {
  const compiled = [];
  for (const target of options.targets || []) {
    assertCompilableSelectorPatterns(target.sourceSpecifierPatterns);
    assertCompilableSelectorPatterns(target.calleeNamePatterns);
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
