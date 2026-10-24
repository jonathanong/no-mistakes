"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapSequence: unwrap } = require("./async-ast");
const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_NAMES = new Set(["test", "it", "describe"]);
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function staticName(node, computed) {
  node = unwrapExpression(node);
  if (!node) return null;
  if (computed && node.type === "Identifier") return null;
  if (!computed && typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node);
}

function constInit(node, context, seen) {
  if (!context || node?.type !== "Identifier" || seen.has(node.name)) return null;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.parent?.kind !== "const" || !def.node.init) return null;
  if (def.node.id.type !== "Identifier") return null;
  seen.add(node.name);
  return def.node.init;
}

function chainInfo(node, context, seen = new Set()) {
  let current = unwrap(node);
  const init = constInit(current, context, seen);
  if (init) current = unwrap(init);
  if (current?.type === "MemberExpression") {
    const forwarded = staticName(current.property, current.computed);
    if (forwarded === "call" || forwarded === "apply") {
      current = unwrap(current.object);
    }
  }
  const names = [];
  while (current?.type === "MemberExpression" || current?.type === "CallExpression") {
    if (current.type === "CallExpression") {
      current = unwrap(current.callee);
      continue;
    }
    names.push(staticName(current.property, current.computed));
    current = unwrap(current.object);
    const init = constInit(current, context, seen);
    if (init) current = unwrap(init);
  }
  return { names, root: current?.type === "Identifier" ? current : null };
}

function hasSequentialMember(node) {
  return chainInfo(node, null).names.includes("sequential");
}

function hasSequentialOption(node) {
  return (node.arguments || []).slice(1).some((argument) => {
    const value = unwrap(argument);
    return (
      value?.type === "ObjectExpression" &&
      value.properties.some((property) => {
        if (property.type !== "Property") return false;
        const key = staticName(property.key, property.computed);
        const option = unwrapExpression(property.value);
        return key === "sequential" && option?.type === "Literal" && option.value === true;
      })
    );
  });
}

function acceptsRunner(root, context, namespaces) {
  if (!root || root.type !== "Identifier") return false;
  const variable = resolveVariable(root, context);
  if (!variable || variable.defs.length === 0) return true;
  if (variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  const scope = variable.scope;
  const scopeType = scope?.type;
  const nested = Boolean(
    scopeType &&
    scopeType !== "module" &&
    scopeType !== "global" &&
    !(scopeType === "function" && scope.block?.type === "Program"),
  );
  if (def.type === "Parameter" || (nested && def.type !== "ImportBinding")) return false;
  if (def.type !== "ImportBinding") return true;
  if (def.node?.type === "ImportNamespaceSpecifier") return namespaces.has(root.name);
  if (def.node?.type === "ImportDefaultSpecifier") return true;
  const imported = staticName(def.node?.imported, false);
  const source = def.parent?.source?.value;
  if (source && source !== "vitest" && TEST_NAMES.has(imported) && imported !== root.name)
    return false;
  return true;
}

function isUnshadowedNamespace(identifier, context, namespaces) {
  if (!namespaces.has(identifier?.name)) return false;
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length === 0) return true;
  if (variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  return def.type === "ImportBinding" && def.node?.type === "ImportNamespaceSpecifier";
}

function isVitestRequire(node) {
  let call = node;
  while (call?.type === "ChainExpression") call = call.expression;
  if (call?.type !== "CallExpression") return false;
  let callee = call.callee;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  return (
    callee?.type === "Identifier" &&
    callee.name === "require" &&
    literalString(call.arguments?.[0]) === "vitest"
  );
}

function destructuredTest(node, context, namespaces) {
  if (node?.type !== "Identifier") return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.parent?.kind !== "const") return false;
  if (def.node.id?.type !== "ObjectPattern") return false;
  const fromNamespace =
    def.node.init?.type === "Identifier" &&
    isUnshadowedNamespace(def.node.init, context, namespaces);
  if (!fromNamespace && !isVitestRequire(def.node.init)) return false;
  return def.node.id.properties.some((property) => {
    if (property.type !== "Property") return false;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    const key = property.computed ? literalString(property.key) : staticName(property.key, false);
    return value?.type === "Identifier" && value.name === node.name && TEST_NAMES.has(key);
  });
}

function isSequentialCall(node, context, roots, namespaces) {
  const matches = (info) => {
    if (!info.root) return false;
    const accepted = acceptsRunner(info.root, context, namespaces);
    const known =
      (accepted && roots.has(info.root.name)) ||
      (accepted &&
        namespaces.has(info.root.name) &&
        info.names.some((name) => TEST_NAMES.has(name))) ||
      destructuredTest(info.root, context, namespaces);
    return known && (info.names.includes("sequential") || hasSequentialOption(node));
  };
  // A const alias of a non-runner still keeps the local runner name.
  return matches(chainInfo(node.callee, context)) || matches(chainInfo(node.callee, null));
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: { description: "disallow Vitest sequential test modifiers", recommended: false },
      schema: [],
      messages: { sequential: "Use parallel tests instead of .sequential." },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "no-vitest-sequential");
      let usesVitestImport = false;
      const roots = new Set(TEST_NAMES);
      const namespaces = new Set();
      return {
        ImportDeclaration(node) {
          if (node.source.value !== "vitest") return;
          usesVitestImport = true;
          for (const specifier of node.specifiers) {
            if (specifier.type === "ImportNamespaceSpecifier") namespaces.add(specifier.local.name);
            if (specifier.type !== "ImportSpecifier") continue;
            const imported = staticName(specifier.imported, false);
            if (TEST_NAMES.has(imported)) roots.add(specifier.local.name);
          }
        },
        CallExpression(node) {
          if (!isTestFile(context.filename) && !usesVitestImport) return;
          if (!isSequentialCall(node, context, roots, namespaces) || suppressed(node)) return;
          const inner = unwrap(node.callee);
          if (
            inner?.type === "CallExpression" &&
            isSequentialCall(inner, context, roots, namespaces)
          )
            return;
          context.report({ node: node.callee, messageId: "sequential" });
        },
      };
    },
  ),
  { __test: { constInit, destructuredTest, hasSequentialMember, hasSequentialOption, staticName } },
);
