"use strict";
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { unwrapExpression } = require("./async-ast");
const { configCall } = require("./vitest-timeout-bindings");
const UNKNOWN = { kind: "unknown" };
function taint(value) {
  if (value.kind === "object")
    return {
      ...value,
      properties: new Map(
        [...value.properties].map(([name, entry]) => [
          name,
          { ...entry, value: taint(entry.value) },
        ]),
      ),
    };
  if (value.kind === "array")
    return {
      ...value,
      entries: value.entries.map((entry) => ({ ...entry, value: taint(entry.value) })),
    };
  return UNKNOWN;
}
function merge(left, right) {
  if (right.kind === "null") return left;
  if (right.kind === "unknown") return taint(left);
  if (left.kind !== "object" || right.kind !== "object") return right;
  const properties = new Map((right.unknown ? taint(left) : left).properties);
  for (const [name, entry] of right.properties) {
    if (entry.value.kind === "null") continue;
    const previous = properties.get(name);
    let value = entry.value;
    if (previous && value.kind === "unknown") value = taint(previous.value);
    if (previous?.value.kind === "object" && value.kind === "object")
      value = merge(previous.value, value);
    if (previous?.value.kind === "array" && value.kind === "array")
      value = { kind: "array", entries: [...previous.value.entries, ...value.entries] };
    properties.set(name, { ...entry, value });
  }
  return { kind: "object", properties };
}
function createEvaluator(context, consumed) {
  const evaluating = new Set();
  const memo = new WeakMap();
  function evaluate(node) {
    if (!node) return UNKNOWN;
    node = unwrapExpression(node);
    if (memo.has(node)) return memo.get(node);
    if (evaluating.has(node)) return UNKNOWN;
    evaluating.add(node);
    const value = resolve(node);
    evaluating.delete(node);
    memo.set(node, value);
    return value;
  }
  function resolve(node) {
    if (node.type === "Literal" && node.value === null) return { kind: "null" };
    if (node.type === "Literal")
      return typeof node.value === "number" ? { kind: "number", value: node.value } : UNKNOWN;
    if (node.type === "UnaryExpression" && (node.operator === "-" || node.operator === "+")) {
      const value = evaluate(node.argument);
      return value.kind === "number"
        ? { kind: "number", value: node.operator === "-" ? -value.value : value.value }
        : UNKNOWN;
    }
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      const def = variable?.defs[0];
      // Only module constants: function-local assignments must not become config defaults.
      return def?.type === "Variable" &&
        def.parent.kind === "const" &&
        variable.scope.type === "module"
        ? evaluate(def.node.init)
        : UNKNOWN;
    }
    if (node.type === "ObjectExpression") {
      const properties = new Map();
      let unknown = false;
      for (const property of node.properties) {
        if (property.type === "SpreadElement") {
          const spread = evaluate(property.argument);
          if (spread.kind === "object") {
            if (spread.unknown) {
              unknown = true;
              for (const [key, entry] of properties)
                properties.set(key, { ...entry, value: taint(entry.value) });
            }
            for (const [key, entry] of spread.properties)
              properties.set(key, { ...entry, origin: property });
          } else if (spread.kind !== "null") {
            unknown = true;
            for (const [key, entry] of properties)
              properties.set(key, { ...entry, value: taint(entry.value) });
          }
          continue;
        }
        if (property.type !== "Property") continue;
        const name = property.computed
          ? property.key.type === "Literal"
            ? String(property.key.value)
            : null
          : (property.key.name ?? String(property.key.value));
        if (name !== null)
          properties.set(name, {
            value: property.kind === "init" ? evaluate(property.value) : UNKNOWN,
            origin: property,
          });
        else {
          unknown = true;
          for (const [key, entry] of properties)
            properties.set(key, { ...entry, value: taint(entry.value) });
        }
      }
      return { kind: "object", properties, unknown };
    }
    if (node.type === "ArrayExpression")
      return {
        kind: "array",
        entries: node.elements.flatMap((origin) => {
          if (!origin) return [];
          if (origin.type === "SpreadElement") {
            const value = evaluate(origin.argument);
            return value.kind === "array"
              ? value.entries.map((entry) => ({ ...entry, origin }))
              : [{ value: UNKNOWN, origin }];
          }
          return [{ value: evaluate(origin), origin }];
        }),
      };
    if (node.type === "ArrowFunctionExpression" || node.type === "FunctionExpression") {
      if (node.body.type !== "BlockStatement") return evaluate(node.body);
      const statements = node.body.body;
      return statements.length === 1 && statements[0].type === "ReturnStatement"
        ? evaluate(statements[0].argument)
        : UNKNOWN;
    }
    if (node.type === "CallExpression") {
      const name = configCall(node, context);
      if (!name) return UNKNOWN;
      consumed.add(node);
      const first = evaluate(node.arguments[0]);
      return name === "mergeConfig" ? merge(first, evaluate(node.arguments[1])) : first;
    }
    return UNKNOWN;
  }
  return evaluate;
}
module.exports = { createEvaluator };
