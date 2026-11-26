"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveImportedValue } = require("./configured-runtime-import");
const { literalString, propertyName } = require("./module-mock-helpers");

// Resolves a factory callee to its exported name. A shadowed or non-import
// binding resolves to null (unjudged). A dynamic member of a known module does too.
function importedFactoryName(sourceCode, callee) {
  const resolved = resolveImportedValue(callee, { sourceCode });
  if (!resolved || resolved.unresolved) return null;
  return resolved.names || [resolved.imported];
}

function staticKey(property) {
  if (property.type !== "Property") return null;
  return property.computed
    ? literalString(unwrapExpression(property.key))
    : propertyName(property.key);
}

function lastProperty(object, name) {
  let found = null;
  for (const property of object.properties) {
    if (staticKey(property) === name) found = property;
  }
  return found;
}

function valueAllowed(sourceCode, property, allowed, allowShorthand) {
  if (property.shorthand) return allowShorthand;
  const value = unwrapExpression(property.value);
  if (value?.type === "Identifier") return allowShorthand;
  if (value?.type !== "CallExpression" && value?.type !== "NewExpression") return true;
  const names = importedFactoryName(sourceCode, value.callee);
  return names === null || names.some((name) => allowed.includes(name));
}

function invalidValueProperties(argument, target, sourceCode) {
  const object = unwrapExpression(argument);
  if (!target.allowedValueCallees || object?.type !== "ObjectExpression") return [];
  const bad = [];
  for (const [name, allowed] of Object.entries(target.allowedValueCallees)) {
    const property = lastProperty(object, name);
    if (property && !valueAllowed(sourceCode, property, allowed, target.allowShorthand)) {
      bad.push(name);
    }
  }
  return bad;
}

function exceedsMaxArguments(target, args) {
  if (!args || args.some((arg) => arg?.type === "SpreadElement")) return false;
  return Number.isInteger(target.maxArguments) && args.length > target.maxArguments;
}

function valueOptionExtras(target) {
  const extras = {};
  if (target.allowedValueCallees && typeof target.allowedValueCallees === "object") {
    extras.allowedValueCallees = target.allowedValueCallees;
    extras.allowShorthand = target.allowShorthand === true;
  }
  if (Number.isInteger(target.maxArguments) && target.maxArguments >= 0) {
    extras.maxArguments = target.maxArguments;
  }
  return extras;
}

module.exports = { exceedsMaxArguments, invalidValueProperties, valueOptionExtras };
