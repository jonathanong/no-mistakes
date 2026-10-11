"use strict";
const {
  resolveVariable,
  isReassigned,
  recordObjectPatternBindings,
} = require("./async-target-bindings");
const { importSpecifierName } = require("./test-no-shared-state-callees");
const { unwrapExpression, isFunction } = require("./async-ast");
const { createCallbackBindings } = require("./timeout-callback-bindings");
const HOOKS = new Set([
  "beforeAll",
  "beforeEach",
  "afterAll",
  "afterEach",
  "aroundAll",
  "aroundEach",
  "onTestFinished",
  "onTestFailed",
]);
const CONFIG = new Set(["defineConfig", "defineProject", "mergeConfig"]);
const TESTS = new Set(["it", "test", "describe", "suite"]);
const CHAIN = new Set("skip only todo concurrent sequential fails shuffle".split(" "));
const BUILDERS = new Set(["each", "for", "skipIf", "runIf", "extend", "override", "scoped"]);
function createBindingResolver(context) {
  const active = new Set();
  const calls = [];
  const { callback, staticName, member } = createCallbackBindings(context);
  function imported(name, source, root) {
    if ((source === "vitest/config" || source === "vite") && CONFIG.has(name))
      return { kind: "config", name, root };
    if (source !== "vitest") return null;
    if (HOOKS.has(name)) return { kind: "hook", name, root };
    if (TESTS.has(name)) return { kind: "test", name, root };
    if (name === "vi" || name === "vitest") return { kind: "runtime", root };
    return null;
  }
  function property(owner, name) {
    if (!owner) return null;
    if (
      (name === "call" || name === "apply") &&
      ["test", "hook", "setter", "unknown"].includes(owner.kind)
    )
      return { kind: "invocation", target: owner, mode: name, root: owner.root };
    if (owner.kind === "namespace")
      return name === null
        ? { kind: "unknown", root: owner.root }
        : imported(name, owner.source, owner.root);
    if (name === null && new Set(["test", "runtime", "context"]).has(owner.kind))
      return { kind: "unknown", root: owner.root };
    if (owner.kind === "runtime" && name === "setConfig")
      return { kind: "setter", root: owner.root };
    if (owner.kind === "context" && (name === "onTestFinished" || name === "onTestFailed"))
      return { kind: "hook", name, root: owner.root };
    if (owner.kind !== "test") return null;
    if (HOOKS.has(name)) return { kind: "hook", name, root: owner.root };
    if (CHAIN.has(name)) return { ...owner };
    if (name === "describe" || name === "suite") return { ...owner, name };
    if (BUILDERS.has(name)) return { kind: "builder", name, owner, root: owner.root };
    return null;
  }
  function resolve(node) {
    node = unwrapExpression(node);
    if (!node) return null;
    if (node.type === "MemberExpression") return property(resolve(node.object), member(node));
    if (node.type === "TaggedTemplateExpression") {
      const api = resolve(node.tag);
      return api?.kind === "builder" && api.name === "each"
        ? { ...api.owner, table: "each" }
        : null;
    }
    if (node.type === "CallExpression") {
      const callee = unwrapExpression(node.callee);
      if (callee.type === "MemberExpression" && member(callee) === "bind") {
        const target = resolve(callee.object),
          receiver = resolve(node.arguments[0]);
        // Runtime methods require the proven vi/vitest receiver. Other SDK registrations allow their own owner.
        const valid =
          target?.kind === "setter"
            ? receiver?.kind === "runtime" && receiver.root === target.root
            : target?.kind === "hook" && receiver?.root === target.root;
        if (!valid)
          return target && new Set(["setter", "hook", "test"]).has(target.kind)
            ? { kind: "unknown", root: target.root }
            : null;
        return node.arguments.length > 1 ? { kind: "unknown", root: target.root } : target;
      }
      const api = resolve(callee);
      if (api?.kind !== "builder") return null;
      return {
        ...api.owner,
        table: api.name === "each" || api.name === "for" ? api.name : api.owner.table,
      };
    }
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (!variable?.defs.length) return imported(node.name, "vitest", node.name);
    if (active.has(variable)) return null;
    active.add(variable);
    try {
      const def = variable.defs[0];
      if (def.type === "ImportBinding") {
        if (
          def.node.type === "ImportNamespaceSpecifier" &&
          new Set(["vitest", "vitest/config", "vite"]).has(def.parent.source.value)
        )
          return { kind: "namespace", source: def.parent.source.value, root: variable };
        return imported(importSpecifierName(def.node), def.parent.source.value, variable);
      }
      if (def.type === "Parameter") {
        const fn = def.node;
        if (!isFunction(fn)) return null;
        const registrations = calls
          .filter((call) =>
            call.arguments.some(
              (arg) =>
                callback(arg) === fn ||
                (arg.type === "ArrayExpression" &&
                  arg.elements.some((element) => callback(element) === fn)),
            ),
          )
          .map((call) => invocation(call))
          .filter(
            (entry) =>
              entry.api?.kind === "test" && entry.args?.some((arg) => callback(arg) === fn),
          );
        if (!registrations.length) return null;
        const api = registrations[0].api;
        if (
          registrations.some(
            (entry) =>
              entry.api.name === "describe" ||
              entry.api.name === "suite" ||
              entry.api.table === "each" ||
              entry.api.table !== api.table,
          )
        )
          return null;
        const parameter = fn.params[api.table === "for" ? 1 : 0],
          owner = { kind: "context", root: fn };
        if (parameter?.type === "Identifier" && parameter.name === node.name) return owner;
        let result = null;
        if (parameter?.type === "ObjectPattern")
          recordObjectPatternBindings(parameter, owner, (id, source, name) => {
            if (id?.name === node.name) result = property(source, name);
          });
        return result;
      }
      if (def.type !== "Variable" || def.parent.kind !== "const" || isReassigned(node, context))
        return null;
      if (def.node.id.type === "Identifier") return resolve(def.node.init);
      let result = null;
      if (def.node.id.type === "ObjectPattern")
        recordObjectPatternBindings(def.node.id, resolve(def.node.init), (id, source, name) => {
          if (id?.name === node.name) result = property(source, name);
        });
      return result;
    } finally {
      active.delete(variable);
    }
  }
  function invocation(node) {
    const api = resolve(node.callee);
    if (api?.kind !== "invocation") return { api, args: node.arguments };
    if (api.target.kind === "setter") {
      const receiver = resolve(node.arguments[0]);
      if (receiver?.kind !== "runtime" || receiver.root !== api.target.root)
        return { api: { kind: "unknown", root: api.root }, args: [] };
    }
    if (api.mode === "call")
      return node.arguments.slice(1).some((arg) => arg.type === "SpreadElement")
        ? { api: { kind: "unknown", root: api.root }, args: [] }
        : { api: api.target, args: node.arguments.slice(1) };
    const array = unwrapExpression(node.arguments[1]);
    return array?.type === "ArrayExpression" &&
      array.elements.every((arg) => arg && arg.type !== "SpreadElement")
      ? { api: api.target, args: array.elements }
      : { api: { kind: "unknown", root: api.root }, args: [] };
  }
  const opaqueCallback = (node) => !callback(node) && Boolean(callback(node, new Set(), true));
  return Object.assign(resolve, {
    staticName,
    callback,
    opaqueCallback,
    invocation,
    record: (node) => calls.push(node),
  });
}
module.exports = { createBindingResolver };
