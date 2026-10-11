"use strict";
const { unwrapExpression, isFunction } = require("./async-ast");
const {
  resolveVariable,
  isReassigned,
  recordObjectPatternBindings,
} = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");
const HOOKS = new Set(["beforeAll", "beforeEach", "afterAll", "afterEach"]);
const CHAINS = new Set(["only", "skip", "fixme", "fail", "describe", "parallel", "serial"]);

function createBindings(context, modules, exportRoles = {}, calls = []) {
  const active = new Set();
  function callback(node, seen = new Set(), allowPartial = false) {
    node = unwrapExpression(node);
    if (isFunction(node)) return node;
    if (
      node?.type === "CallExpression" &&
      node.callee.type === "MemberExpression" &&
      member(node.callee) === "bind"
    ) {
      const partial =
        node.arguments.length > 1 || node.arguments.some((arg) => arg.type === "SpreadElement");
      return partial && !allowPartial ? null : callback(node.callee.object, seen, allowPartial);
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context),
      def = variable?.defs[0];
    if (seen.has(variable)) return null;
    if (def?.type === "FunctionName")
      return variable.references.some((ref) => ref.isWrite() && !ref.init) ? null : def.node;
    if (isReassigned(node, context)) return null;
    if (def?.type !== "Variable" || def.parent.kind !== "const") return null;
    return callback(def.node.init, new Set([...seen, variable]), allowPartial);
  }
  function registrationFor(fn) {
    return calls.filter(
      (call) =>
        call.arguments.some((arg) => callback(arg) === fn) && registration(resolve(call.callee)),
    );
  }
  function name(node, seen = new Set()) {
    node = unwrapExpression(node);
    if (!node) return null;
    const literal = literalString(node);
    if (literal !== null) return literal;
    if (node.type === "BinaryExpression" && node.operator === "+") {
      const left = name(node.left, seen),
        right = name(node.right, seen);
      return left === null || right === null ? null : left + right;
    }
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context),
      def = variable?.defs[0];
    if (
      seen.has(variable) ||
      def?.type !== "Variable" ||
      def.parent.kind !== "const" ||
      isReassigned(node, context)
    )
      return null;
    return name(def.node.init, new Set([...seen, variable]));
  }
  const member = (node) => (node.computed ? name(node.property) : node.property.name);
  function property(owner, key) {
    if (!owner) return null;
    if (owner.kind === "opaque") return owner;
    if (owner.kind === "namespace") {
      const role =
        owner.source === "@playwright/test" || ["test", "defineConfig", "mergeTests"].includes(key)
          ? undefined
          : exportRoles[owner.source]?.[key];
      if (role === "assertion" || role === "ordinary") return null;
      if (role === "opaque") return { kind: "opaque", root: owner.root };
      if (key === "test" || (key === "default" && owner.source === "@playwright/test"))
        return { kind: "test", root: owner.root, path: [] };
      if (key === "defineConfig" || key === "mergeTests") return { kind: key, root: owner.root };
      return owner.source !== "@playwright/test" ? { kind: "opaque", root: owner.root } : null;
    }
    if (owner.kind === "test") return { kind: "method", owner, key };
    if (owner.kind === "method" && CHAINS.has(owner.key))
      return property(
        { kind: "test", root: owner.owner.root, path: [...owner.owner.path, owner.key] },
        key,
      );
    if (owner.kind === "info" && (key === "setTimeout" || key === "slow" || key === null))
      return { kind: "method", owner, key };
    return null;
  }
  function registration(api) {
    return (
      api?.kind === "test" ||
      (api?.kind === "method" && (CHAINS.has(api.key) || HOOKS.has(api.key)))
    );
  }
  function resolve(node) {
    node = unwrapExpression(node);
    if (!node) return null;
    if (node.type === "MemberExpression") return property(resolve(node.object), member(node));
    if (node.type === "CallExpression") {
      const callee = unwrapExpression(node.callee);
      if (callee.type === "MemberExpression" && member(callee) === "bind") {
        const target = resolve(callee.object),
          receiver = resolve(node.arguments[0]);
        if (target?.kind !== "method") return null;
        if (
          receiver?.kind !== target.owner.kind ||
          receiver.root !== target.owner.root ||
          JSON.stringify(receiver.path) !== JSON.stringify(target.owner.path)
        )
          return {
            kind: "partial",
            target,
            receiverUnresolved: true,
            boundArguments: node.arguments.slice(1),
          };
        // Pre-bound arguments precede later call arguments. Never check the later value as effective.
        return node.arguments.length > 1
          ? { kind: "partial", target, boundArguments: node.arguments.slice(1) }
          : target;
      }
      const api = resolve(callee);
      if (api?.kind === "method" && api.owner.kind === "test") {
        if (api.key === "info") return { kind: "info", root: api.owner.root };
        if (api.key === "extend") return { kind: "test", root: node, path: [] };
      }
      if (api?.kind === "opaque") return api;
      if (api?.kind === "mergeTests")
        return node.arguments.length && node.arguments.every((arg) => resolve(arg)?.kind === "test")
          ? { kind: "test", root: node, path: [] }
          : { kind: "opaque", root: node };
      return null;
    }
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (!variable || active.has(variable)) return null;
    active.add(variable);
    try {
      const def = variable.defs[0];
      if (def?.type === "ImportBinding" && modules.has(def.parent.source.value)) {
        if (def.node.type === "ImportNamespaceSpecifier")
          return { kind: "namespace", root: variable, source: def.parent.source.value };
        const imported = def.node.imported?.name ?? def.node.imported?.value;
        const key =
          def.parent.source.value === "@playwright/test" &&
          (def.node.type === "ImportDefaultSpecifier" || imported === "default")
            ? "test"
            : imported;
        const role =
          def.parent.source.value === "@playwright/test" ||
          ["test", "defineConfig", "mergeTests"].includes(key)
            ? undefined
            : exportRoles[def.parent.source.value]?.[key];
        if (role === "assertion" || role === "ordinary") return null;
        if (role === "opaque") return { kind: "opaque", root: variable };
        if (key === "test") return { kind: "test", root: variable, path: [] };
        if (key === "defineConfig" || key === "mergeTests") return { kind: key, root: variable };
        if (def.parent.source.value !== "@playwright/test")
          return { kind: "opaque", root: variable };
      }
      if (def?.type === "Parameter") {
        const fn = def.node;
        if (!isFunction(fn)) return null;
        const registrations = registrationFor(fn);
        if (!registrations.length) return null;
        const param = fn.params[1];
        if (
          registrations.some((call) => {
            const api = resolve(call.callee);
            return (
              api?.kind === "method" &&
              (api.key === "describe" || api.owner.path?.includes("describe"))
            );
          })
        )
          return null;
        const info = { kind: "info", root: fn };
        if (param?.type === "Identifier" && param.name === node.name) return info;
        let result = null;
        if (param?.type === "ObjectPattern")
          recordObjectPatternBindings(param, info, (id, source, key) => {
            if (id?.name === node.name) result = property(source, key);
          });
        return result;
      }
      if (def?.type !== "Variable" || def.parent.kind !== "const" || isReassigned(node, context))
        return null;
      if (def.node.id.type === "Identifier") return resolve(def.node.init);
      let result = null;
      if (def.node.id.type === "ObjectPattern")
        recordObjectPatternBindings(def.node.id, resolve(def.node.init), (id, source, key) => {
          if (id?.name === node.name) result = property(source, key);
        });
      return result;
    } finally {
      active.delete(variable);
    }
  }
  const opaqueCallback = (node) => !callback(node) && Boolean(callback(node, new Set(), true));
  return { resolve, registration, registrationFor, opaqueCallback, member, hooks: HOOKS };
}
module.exports = { createBindings };
