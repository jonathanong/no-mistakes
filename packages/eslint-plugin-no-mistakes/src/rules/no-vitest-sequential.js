"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapSequence: unwrap } = require("./async-ast");
const { requireSource } = require("./async-target-sources");
const { literalString, logicalCall } = require("./logical-call");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const {
  destructuredFromNamespace,
  recordImportEquals,
  recordRequire,
  recordVitestImport,
  unresolvedSequential,
} = require("./no-vitest-sequential-loads");
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
      const init = constInit(current, context, seen);
      if (init) current = unwrap(init);
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
  // Opaque forwarding must not erase options already visible to the legacy check.
  const args =
    node.type === "CallExpression"
      ? (logicalCall(node)?.arguments ?? node.arguments)
      : node.arguments;
  return (args || []).slice(1).some((argument) => {
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

function destructuredTest(node, context, namespaces) {
  if (node?.type !== "Identifier") return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.node?.parent?.kind !== "const") return false;
  if (def.node.id?.type !== "ObjectPattern") return false;
  const fromNamespace = destructuredFromNamespace(def.node.init, context, namespaces);
  if (!fromNamespace && requireSource(def.node.init, context) !== "vitest") return false;
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
      schema: [{ type: "object", properties: { unanalyzable: UNANALYZABLE_SCHEMA } }],
      messages: {
        sequential: "Use parallel tests instead of .sequential.",
        unanalyzable:
          "Cannot tell whether {{subject}} runs tests sequentially. Use a static vitest import.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "no-vitest-sequential");
      const state = {
        namespaces: new Set(),
        opaque: new Set(),
        roots: new Set(TEST_NAMES),
        usesVitest: false,
      };
      return {
        ImportDeclaration(node) {
          recordVitestImport(node, state);
        },
        TSImportEqualsDeclaration(node) {
          recordImportEquals(node, state);
        },
        VariableDeclarator(node) {
          recordRequire(node, state, context);
        },
        CallExpression(node) {
          if (!isTestFile(context.filename) && !state.usesVitest) return;
          if (!isSequentialCall(node, context, state.roots, state.namespaces)) {
            if (
              (hasSequentialMember(node.callee) || hasSequentialOption(node)) &&
              unresolvedSequential(node.callee, context) &&
              !suppressed(node)
            ) {
              reportUnanalyzable(context, node.callee, "this call");
            }
            return;
          }
          if (suppressed(node)) return;
          const inner = unwrap(node.callee);
          if (
            inner?.type === "CallExpression" &&
            isSequentialCall(inner, context, state.roots, state.namespaces)
          )
            return;
          context.report({ node: node.callee, messageId: "sequential" });
        },
      };
    },
  ),
  { __test: { constInit, destructuredTest, hasSequentialMember, hasSequentialOption, staticName } },
);
