"use strict";

const { isFunction, unwrapExpression } = require("./async-ast");
const {
  isLocalRequire,
  isReassigned,
  resolveVariable,
  stableConstInitializer,
} = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");

const MODIFIERS = new Set(["only", "skip", "fixme", "fail"]);
const HOOKS = new Set(["beforeAll", "beforeEach", "afterAll", "afterEach"]);

function createBrowserBindings(context, modules) {
  function staticString(node, seen = new Set()) {
    node = unwrapExpression(node);
    const literal = literalString(node);
    if (literal !== null) return literal;
    const init = stableConstInitializer(node, context, seen);
    return init ? staticString(init, seen) : null;
  }

  function memberKey(node) {
    if (!node.computed && node.property.type === "Identifier") return node.property.name;
    return staticString(node.property);
  }

  function propertyKey(property) {
    if (!property.computed) {
      return property.key.type === "Identifier" ? property.key.name : literalString(property.key);
    }
    return staticString(property.key);
  }

  function requireSource(node) {
    node = unwrapExpression(node);
    if (
      node?.type === "CallExpression" &&
      node.callee.type === "Identifier" &&
      node.callee.name === "require" &&
      !isLocalRequire(node.callee, context)
    ) {
      return literalString(node.arguments[0]);
    }
    return null;
  }

  function patternKeyForName(pattern, name) {
    const match = pattern.properties.find((prop) => {
      if (prop.type !== "Property") return false;
      const target = prop.value.type === "AssignmentPattern" ? prop.value.left : prop.value;
      return target.type === "Identifier" && target.name === name;
    });
    return match ? propertyKey(match) : null;
  }

  function memberApi(owner, key) {
    if (!owner) return null;
    if (owner.kind === "dynamic") return owner;
    if (owner.kind === "namespace") {
      if (key === null) return { kind: "dynamic" };
      if (key === "test" || (key === "default" && owner.source === "@playwright/test")) {
        return { kind: "test" };
      }
      return key === "mergeTests" ? { kind: "mergeTests" } : null;
    }
    if (owner.kind === "describe") return owner;
    if (owner.kind === "test" || owner.kind === "modifier") {
      if (key === null) return { kind: "dynamic" };
      if (key === "describe") return { kind: "describe" };
      if (MODIFIERS.has(key)) return { kind: "modifier", key };
      if (HOOKS.has(key)) return { kind: "hook", key };
      if (owner.kind === "test" && key === "extend") return { kind: "extend" };
    }
    return null;
  }

  function resolveRegistrar(node, seen = new Set()) {
    node = unwrapExpression(node);
    if (node.type === "MemberExpression") {
      return memberApi(resolveRegistrar(node.object, seen), memberKey(node));
    }
    if (node.type === "CallExpression") {
      const cjs = requireSource(node);
      if (cjs && modules.has(cjs)) return { kind: "namespace", source: cjs };
      const api = resolveRegistrar(node.callee, seen);
      return api?.kind === "extend" || api?.kind === "mergeTests" ? { kind: "test" } : null;
    }
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
    seen.add(variable);
    const def = variable.defs[0];
    if (def.type === "ImportBinding" && def.parent.type === "ImportDeclaration") {
      if (def.node.importKind === "type" || def.parent.importKind === "type") return null;
      const source = def.parent.source.value;
      if (!modules.has(source)) return null;
      if (def.node.type === "ImportNamespaceSpecifier") return { kind: "namespace", source };
      const imported =
        def.node.type === "ImportDefaultSpecifier"
          ? "default"
          : (def.node.imported.name ?? def.node.imported.value);
      return memberApi({ kind: "namespace", source }, imported);
    }
    if (def.type !== "Variable" || isReassigned(node, context) || !def.node.init) return null;
    if (def.node.id.type === "Identifier") return resolveRegistrar(def.node.init, seen);
    return def.node.id.type === "ObjectPattern"
      ? memberApi(resolveRegistrar(def.node.init, seen), patternKeyForName(def.node.id, node.name))
      : null;
  }

  function resolveCallback(node, seen = new Set()) {
    node = unwrapExpression(node);
    if (isFunction(node)) return { kind: "fn", fn: node };
    if (
      node.type === "CallExpression" &&
      node.callee.type === "MemberExpression" &&
      memberKey(node.callee) === "bind"
    ) {
      if (node.arguments.length > 1 || node.arguments.some((arg) => arg.type === "SpreadElement")) {
        return { kind: "opaque", node };
      }
      return resolveCallback(node.callee.object, seen);
    }
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      if (variable && !seen.has(variable) && variable.defs.length === 1) {
        const def = variable.defs[0];
        if (
          def.type === "FunctionName" &&
          !variable.references.some((ref) => ref.isWrite() && !ref.init)
        ) {
          return { kind: "fn", fn: def.node };
        }
        if (
          def.type === "Variable" &&
          def.node.id.type === "Identifier" &&
          def.node.init &&
          !isReassigned(node, context)
        ) {
          seen.add(variable);
          return resolveCallback(def.node.init, seen);
        }
      }
    }
    return { kind: "opaque", node };
  }

  function callbackSlot(slot, kind) {
    if (slot.type === "SpreadElement") {
      return { kind: "opaque", node: slot, reason: `spread ${kind} arguments` };
    }
    const resolved = resolveCallback(slot);
    return resolved.kind === "fn"
      ? resolved
      : { kind: "opaque", node: slot, reason: `unresolved ${kind} callback` };
  }

  function extractCallTarget(callNode) {
    const api = resolveRegistrar(callNode.callee);
    if (!api) return null;
    if (api.kind === "dynamic") {
      return { kind: "opaque", node: callNode, reason: "dynamic Playwright test method" };
    }
    const args = callNode.arguments;
    if (api.kind === "test" || (api.kind === "modifier" && api.key === "only")) {
      if (args.length === 1 && args[0].type === "SpreadElement") {
        return { kind: "opaque", node: args[0], reason: "spread test arguments" };
      }
      return args.length < 2 ? null : callbackSlot(args[args.length - 1], "test");
    }
    if (api.kind === "hook") {
      return args.length === 0 ? null : callbackSlot(args[args.length - 1], "hook");
    }
    if (api.kind === "modifier") {
      if (args.length === 0) return null;
      if (args[0].type === "SpreadElement") {
        return { kind: "opaque", node: args[0], reason: "spread test arguments" };
      }
      const first = resolveCallback(args[0]);
      if (first.kind === "fn") return first;
      if (args.length < 2) return null;
      const slot = args[args.length - 1];
      if (slot.type === "SpreadElement") {
        return { kind: "opaque", node: slot, reason: "spread test arguments" };
      }
      const resolved = resolveCallback(slot);
      if (resolved.kind === "fn") return resolved;
      const last = unwrapExpression(slot);
      return staticString(last) !== null || last.type === "TemplateLiteral"
        ? null
        : { kind: "opaque", node: slot, reason: "unresolved test callback" };
    }
    return null;
  }

  return { extractCallTarget, memberKey, propertyKey };
}

module.exports = { createBrowserBindings };
