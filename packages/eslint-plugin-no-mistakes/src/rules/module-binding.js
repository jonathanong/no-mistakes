"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  isLocalRequire,
  isReassigned,
  resolveVariable,
  staticImportSource,
} = require("./async-target-bindings");
const { importSpecifierName, requireSource } = require("./async-target-sources");
const { literalString } = require("./module-mock-helpers");

const PLAYWRIGHT_TEST = "@playwright/test";

function staticKey(property) {
  const key = unwrapExpression(property?.key);
  if (!property || property.type !== "Property") return null;
  if (property.computed && key?.type === "Identifier") return null;
  if (!property.computed && typeof property.key?.name === "string") return property.key.name;
  return literalString(key);
}

function memberKey(node) {
  const target = unwrapExpression(node);
  if (target?.type !== "MemberExpression") return undefined;
  const property = unwrapExpression(target.property);
  if (target.computed && property?.type === "Identifier") return null;
  if (!target.computed && typeof target.property?.name === "string") return target.property.name;
  return literalString(property);
}

function requireFact(node, context) {
  const expression = unwrapExpression(node);
  const callee = unwrapExpression(expression?.callee);
  if (
    expression?.type !== "CallExpression" ||
    callee?.type !== "Identifier" ||
    callee.name !== "require"
  ) {
    return null;
  }
  const source = requireSource(expression, context);
  if (source) return { status: "namespace", source };
  if (isLocalRequire(callee, context)) return null;
  return { status: "opaque", source: null, reason: "dynamic require" };
}

function fromImportDef(def) {
  const source = staticImportSource(def);
  if (!source) return null;
  const node = def.node;
  if (node.type === "ImportNamespaceSpecifier" || node.type === "TSImportEqualsDeclaration") {
    return { status: "namespace", source };
  }
  if (node.type === "ImportDefaultSpecifier") {
    return { status: "export", source, exported: "default" };
  }
  if (node.type !== "ImportSpecifier") return null;
  const exported = node.imported ? importSpecifierName(node) : null;
  return exported
    ? { status: "export", source, exported }
    : { status: "opaque", source, reason: "unreadable import name" };
}

function applyMember(fact, key) {
  if (!fact || fact.status === "opaque") return fact;
  if (key == null) return { status: "opaque", source: fact.source, reason: "dynamic member" };
  if (fact.status !== "namespace") return null;
  return { status: "export", source: fact.source, exported: key };
}

function patternKeyFor(pattern, name) {
  if (pattern?.type !== "ObjectPattern") return undefined;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type === "Identifier" && value.name === name) return staticKey(property);
  }
  return undefined;
}

function resolveModuleBinding(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (!node) return null;
  const required = requireFact(node, context);
  if (required) return required;
  if (node.type === "MemberExpression") {
    const key = memberKey(node);
    return key === undefined
      ? null
      : applyMember(resolveModuleBinding(node.object, context, seen), key);
  }
  if (node.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  const imported = fromImportDef(def);
  if (imported) return imported;
  if (def.type !== "Variable" || !def.node) return null;
  seen.add(variable);
  if (isReassigned(node, context)) return null;
  if (def.node.id?.type === "Identifier") return resolveModuleBinding(def.node.init, context, seen);
  const key = patternKeyFor(def.node.id, node.name);
  if (key === undefined) return null;
  return applyMember(resolveModuleBinding(def.node.init, context, seen), key);
}

function playwrightRequire(node, context) {
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") {
    if (memberKey(current) == null) break;
    current = unwrapExpression(current.object);
  }
  return requireSource(current, context) === PLAYWRIGHT_TEST;
}

function admitsPlaywrightModule(node, context) {
  if (!node) return false;
  if (node.type === "ImportDeclaration") return node.source?.value === PLAYWRIGHT_TEST;
  if (node.type === "TSImportEqualsDeclaration") {
    const reference = node.moduleReference;
    return (
      reference?.type === "TSExternalModuleReference" &&
      literalString(unwrapExpression(reference.expression)) === PLAYWRIGHT_TEST
    );
  }
  if (node.type === "VariableDeclarator") return playwrightRequire(node.init, context);
  if (node.type === "ExpressionStatement") return playwrightRequire(node.expression, context);
  return playwrightRequire(node, context);
}

function reportUnanalyzable(option) {
  return (option?.unanalyzable ?? "finding") === "finding";
}

module.exports = {
  PLAYWRIGHT_TEST,
  admitsPlaywrightModule,
  reportUnanalyzable,
  requireFact,
  resolveModuleBinding,
};
