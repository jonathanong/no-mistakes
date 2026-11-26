"use strict";

const { unwrapExpression } = require("./async-ast");
const { literalString, propertyName } = require("./module-mock-helpers");

function findVariable(sourceCode, node, name) {
  for (let scope = sourceCode.getScope(node); scope; scope = scope.upper) {
    const variable = scope.set.get(name);
    if (variable) return variable;
  }
  return null;
}

function importBinding(sourceCode, node, name) {
  const def = findVariable(sourceCode, node, name)?.defs[0];
  if (def?.type !== "ImportBinding" || def.parent?.importKind === "type") return null;
  return def.node;
}

function memberName(member) {
  return member.computed
    ? literalString(unwrapExpression(member.property))
    : propertyName(member.property);
}

// Resolves a factory callee to its exported name through named, renamed,
// default, and namespace imports. A shadowed or non-import binding resolves to null (unjudged).
function importedFactoryName(sourceCode, callee) {
  const node = unwrapExpression(callee);
  if (node?.type === "Identifier") {
    const spec = importBinding(sourceCode, node, node.name);
    if (spec?.type === "ImportSpecifier") {
      return propertyName(spec.imported);
    }
    return spec?.type === "ImportDefaultSpecifier" ? node.name : null;
  }
  if (node?.type !== "MemberExpression" || node.object.type !== "Identifier") return null;
  const spec = importBinding(sourceCode, node, node.object.name);
  if (spec?.type === "ImportNamespaceSpecifier" || spec?.type === "ImportDefaultSpecifier") {
    return memberName(node);
  }
  return null;
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
  const name = importedFactoryName(sourceCode, value.callee);
  return name === null || allowed.includes(name);
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
