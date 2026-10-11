"use strict";
const { unwrapExpression, isFunction } = require("./async-ast");
const { resolveVariable, isReassigned } = require("./async-target-bindings");
function createCarriers(context, evaluate, check, max, fixtureMax) {
  function field(value, node, name = "timeout", cap = max) {
    if (value.kind !== "object") {
      check({ kind: "unknown" }, node, name, cap);
      return undefined;
    }
    const entry = value.properties.get("timeout");
    if (entry) check(entry.value, entry.origin, name, cap);
    else if (value.unknown) check({ kind: "unknown" }, node, name, cap);
    return entry?.value;
  }
  function config(value, node) {
    if (value.kind !== "object") {
      check({ kind: "unknown" }, node, "config carrier");
      return;
    }
    field(value, node, "project timeout");
    const projects = value.properties.get("projects");
    if (projects) {
      if (projects.value.kind !== "array")
        check({ kind: "unknown" }, projects.origin, "projects carrier");
      else for (const entry of projects.value.entries) config(entry.value, entry.origin);
    } else if (value.unknown) check({ kind: "unknown" }, node, "projects carrier");
  }
  function fixtures(node) {
    const value = evaluate(node);
    if (value.kind !== "object") {
      check({ kind: "unknown" }, node, "fixture carrier", fixtureMax);
      return;
    }
    if (value.unknown) check({ kind: "unknown" }, node, "fixture carrier", fixtureMax);
    for (const entry of value.properties.values()) {
      // Only SDK fixture tuples carry a separate deadline. Ordinary fixture data is not options.
      if (entry.value.kind === "array" && entry.value.entries.length === 2)
        field(
          entry.value.entries[1].value,
          entry.value.entries[1].origin,
          "fixture timeout",
          fixtureMax,
        );
      else if (entry.value.kind === "unknown" && !isFunction(unwrapExpression(entry.origin.value)))
        check({ kind: "unknown" }, entry.origin, "fixture entry carrier", fixtureMax);
    }
  }
  function booleanValue(node, seen = new Set()) {
    node = unwrapExpression(node);
    if (!node) return null;
    if (node.type === "Literal" && typeof node.value === "boolean") return node.value;
    if (node.type === "UnaryExpression" && node.operator === "!") {
      const value = booleanValue(node.argument, seen);
      return value === null ? null : !value;
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
    return booleanValue(def.node.init, new Set([...seen, variable]));
  }
  function slowCondition(node) {
    if (!node.arguments.length) return true;

    // A fixture callback is not evaluated as a config callback: its runtime condition is unknown.
    return booleanValue(node.arguments[0]);
  }
  function configRoots(option, exports, calls, bindings) {
    const checkedConfigs = new Set();
    for (const node of exports) {
      const api = node.type === "CallExpression" ? bindings.resolve(node.callee) : null;
      if (api?.kind === "defineConfig") {
        checkedConfigs.add(node);
        if (node.arguments.length === 1) config(evaluate(node.arguments[0]), node.arguments[0]);
        else check({ kind: "unknown" }, node, "merged config carrier");
      } else if (
        (option.configFiles ?? []).some(
          (file) =>
            context.filename.replace(/\\/g, "/").endsWith("/" + file) || context.filename === file,
        )
      )
        config(evaluate(node), node);
    }
    for (const node of calls) {
      if (checkedConfigs.has(node) || bindings.resolve(node.callee)?.kind !== "defineConfig")
        continue;
      if (node.arguments.length === 1) config(evaluate(node.arguments[0]), node.arguments[0]);
      else check({ kind: "unknown" }, node, "merged config carrier");
    }
  }
  return { field, configRoots, fixtures, slowCondition };
}
module.exports = { createCarriers };
