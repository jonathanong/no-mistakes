"use strict";

const { unwrapExpression } = require("./async-ast");
const { matchesAny } = require("./async-patterns");
const { isReassigned, memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { staticPropertyName } = require("./imported-call-options");
const { bindingIdentifier, hasLocalBinding } = require("./no-global-fetch-outside-helper-bindings");
const { compileCallSelectors, matchesDotted } = require("./required-call-selector-compile");

const GLOBAL_ROOTS = new Set(["globalThis", "window", "self", "global"]);

function staticMemberName(member) {
  if (member?.type !== "MemberExpression") return null;
  if (!member.computed && member.property?.type !== "Identifier") return null;
  return memberPropertyName(member);
}

function staticReceiverPath(node) {
  const parts = [];
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") {
    const name = staticMemberName(current);
    if (!name) return null;
    parts.unshift(name);
    current = unwrapExpression(current.object);
  }
  if (current?.type !== "Identifier") return null;
  parts.unshift(current.name);
  return parts.join(".");
}

function createCallSelectors(context, options) {
  const compiled = compileCallSelectors(options);
  const globalAliases = new Map();
  const memberAliases = new Map();
  const hasTargets = compiled.globals.length > 0 || compiled.members.length > 0;

  function isGlobalRoot(node) {
    const current = unwrapExpression(node);
    return (
      current?.type === "Identifier" &&
      GLOBAL_ROOTS.has(current.name) &&
      !hasLocalBinding(current, context)
    );
  }

  function recordDeclarator(node) {
    if (!node.init || node.id?.type !== "ObjectPattern") return;
    const globalRoot = isGlobalRoot(node.init);
    const receiverPath = staticReceiverPath(node.init);
    if (!globalRoot && !receiverPath) return;
    for (const property of node.id.properties) {
      if (property.type !== "Property") continue;
      const name = staticPropertyName(property);
      const binding = bindingIdentifier(property.value);
      if (!name || !binding || isReassigned(binding, context)) continue;
      const variable = resolveVariable(binding, context);
      if (
        globalRoot &&
        compiled.globals.some((target) => matchesAny(name, target.globalNamePatterns))
      ) {
        globalAliases.set(variable, name);
      }
      if (
        receiverPath &&
        compiled.members.some(
          (target) =>
            matchesAny(name, target.calleeNamePatterns) &&
            matchesDotted(receiverPath, target.receiverPatterns),
        )
      ) {
        memberAliases.set(variable, { receiverPath, calleeName: name });
      }
    }
  }

  function walk(node) {
    if (!node?.type) return;
    if (node.type === "VariableDeclarator") recordDeclarator(node);
    for (const key of context.sourceCode.visitorKeys[node.type] || []) {
      const value = node[key];
      if (Array.isArray(value)) {
        for (const child of value) walk(child);
      } else walk(value);
    }
  }

  function pushGlobals(callee, found) {
    if (callee?.type === "Identifier") {
      const variable = resolveVariable(callee, context);
      const aliased = variable && globalAliases.get(variable);
      const name = aliased || (!hasLocalBinding(callee, context) ? callee.name : null);
      if (!name) return;
      for (const target of compiled.globals) {
        if (!matchesAny(name, target.globalNamePatterns)) continue;
        found.push({ target, resolved: { source: "global", calleeName: name } });
      }
      return;
    }
    const name = staticMemberName(callee);
    if (!name || !isGlobalRoot(callee.object)) return;
    for (const target of compiled.globals) {
      if (!matchesAny(name, target.globalNamePatterns)) continue;
      found.push({ target, resolved: { source: "global", calleeName: name } });
    }
  }

  function pushMembers(callee, found) {
    if (callee?.type === "Identifier") {
      const variable = resolveVariable(callee, context);
      const aliased = variable && memberAliases.get(variable);
      if (!aliased) return;
      for (const target of compiled.members) {
        if (
          !matchesAny(aliased.calleeName, target.calleeNamePatterns) ||
          !matchesDotted(aliased.receiverPath, target.receiverPatterns)
        ) {
          continue;
        }
        found.push({
          target,
          resolved: { source: aliased.receiverPath, calleeName: aliased.calleeName },
        });
      }
      return;
    }
    const terminal = staticMemberName(callee);
    const receiverPath = terminal ? staticReceiverPath(callee.object) : null;
    if (!terminal || !receiverPath) return;
    for (const target of compiled.members) {
      if (
        !matchesAny(terminal, target.calleeNamePatterns) ||
        !matchesDotted(receiverPath, target.receiverPatterns)
      ) {
        continue;
      }
      found.push({ target, resolved: { source: receiverPath, calleeName: terminal } });
    }
  }

  return {
    hasTargets,
    record(program) {
      if (!hasTargets) return;
      walk(program);
    },
    matches(node) {
      if (!hasTargets) return [];
      const found = [];
      const callee = unwrapExpression(node.callee);
      pushGlobals(callee, found);
      pushMembers(callee, found);
      return found;
    },
  };
}

module.exports = { createCallSelectors };
