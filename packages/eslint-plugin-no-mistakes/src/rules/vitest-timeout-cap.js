"use strict";
const { rule, options } = require("../helpers");
const { isFunction, unwrapExpression } = require("./async-ast");
const { resolveVariable, isReassigned } = require("./async-target-bindings");
const { ruleSuppression } = require("../rule-suppression");
const {
  configCall,
  overrideCall,
  recordCall,
  invocationArguments,
  callbackFunction,
  opaqueCallback,
} = require("./vitest-timeout-bindings");
const { createCarriers } = require("./vitest-timeout-carriers");
const { createEvaluator } = require("./vitest-timeout-values");
module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require positive Vitest timeouts and cap project defaults and overrides",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          defaultMax: { type: "number", minimum: 0, exclusiveMinimum: true },
          overrideMax: { type: "number", minimum: 0, exclusiveMinimum: true },
          unknownValues: { enum: ["ignore", "finding"] },
          configRoot: { type: "boolean" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      invalid:
        "Vitest {{name}} must be positive; zero disables the timeout and negative values are invalid.",
      timeout:
        "Vitest {{name}} exceeds {{max}} ms. Fix the slow wait or test setup rather than hiding hangs with a larger timeout.",
      unknown:
        "Vitest {{name}} cannot be statically resolved against the {{max}} ms cap. Use a same-module numeric constant or explicitly ignore unknown values.",
    },
  },
  (context) => {
    const option = options(context);
    for (const name of ["defaultMax", "overrideMax"]) {
      if (option[name] !== undefined && (!Number.isFinite(option[name]) || option[name] <= 0))
        throw new Error(`vitest-timeout-cap ${name} must be a positive finite number`);
    }
    const suppressed = ruleSuppression(context, "vitest-timeout-cap");
    const consumed = new Set();
    const evaluate = createEvaluator(context, consumed);
    const configs = [];
    const overrides = [];
    const exports = [];
    function callback(node, seen = new Set()) {
      if (callbackFunction(node, context)) return true;
      node = unwrapExpression(node);
      if (isFunction(node)) return true;
      if (node?.type !== "Identifier") return false;
      const variable = resolveVariable(node, context),
        def = variable?.defs[0];
      if (
        seen.has(variable) ||
        def?.type !== "Variable" ||
        def.parent.kind !== "const" ||
        isReassigned(node, context)
      )
        return false;
      return callback(def.node.init, new Set([...seen, variable]));
    }
    function check(entry, name, max, origin) {
      const node = origin || entry.origin;
      if (suppressed(node)) return;
      if (entry.value.kind === "number") {
        if (!(entry.value.value > 0))
          context.report({ node, messageId: "invalid", data: { name } });
        else if (entry.value.value > max)
          context.report({ node, messageId: "timeout", data: { name, max } });
      } else if (option.unknownValues === "finding")
        context.report({ node, messageId: "unknown", data: { name, max } });
    }
    const { properties, defaults } = createCarriers(option, check);
    return {
      ExportDefaultDeclaration(node) {
        exports.push(node.declaration);
      },
      CallExpression(node) {
        recordCall(node, context);
        if (configCall(node, context)) configs.push(node);
        overrides.push(node);
      },
      "Program:exit"() {
        // Evaluate exports and outer merges first so overwritten fragment values never report.
        for (const node of exports) {
          const count = consumed.size;
          const value = evaluate(node);
          if (option.configRoot || consumed.size > count) defaults(value, undefined, node);
        }
        const roots = configs.filter(
          (node) =>
            !configs.some(
              (other) =>
                other !== node && other.range[0] < node.range[0] && other.range[1] > node.range[1],
            ),
        );
        for (const node of roots.toReversed())
          if (!consumed.has(node)) defaults(evaluate(node), undefined, node.arguments[0] || node);
        for (const original of overrides) {
          const kind = overrideCall(original, context);
          if (!kind) continue;
          const node = { ...original, arguments: invocationArguments(original, context) };
          const max = option.overrideMax ?? 30000;
          if (kind === "test" || kind === "hook")
            for (const argument of node.arguments)
              if (opaqueCallback(argument, context))
                check(
                  { value: { kind: "unknown" }, origin: argument },
                  "bound callback parameter carrier",
                  max,
                  argument,
                );
          if (kind === "unknown") {
            check({ value: { kind: "unknown" }, origin: node }, "framework API carrier", max, node);
            continue;
          }
          if (kind === "runtime") {
            properties(
              evaluate(node.arguments[0]),
              ["testTimeout", "hookTimeout"],
              max,
              node.arguments[0] || node,
            );
            continue;
          }
          const args = kind === "hook" ? node.arguments.slice(1) : node.arguments.slice(1, 3);
          for (const argument of args) {
            if (callback(argument)) continue;
            const value = evaluate(argument);
            if (value.kind === "object")
              properties(
                value,
                ["timeout"],
                max,
                argument.type === "Identifier" ? argument : undefined,
                "timeout options",
                argument,
              );
            else if (
              kind === "test" &&
              node.arguments.length >= 3 &&
              argument !== node.arguments.at(-1) &&
              value.kind === "unknown"
            )
              properties(value, ["timeout"], max, argument);
          }
          const last = node.arguments.at(-1);
          const minimum = kind === "hook" ? 2 : 3;
          if (!last || node.arguments.length < minimum || callback(last)) continue;
          // An options object followed by a callback is not a trailing timeout.
          if (evaluate(node.arguments.at(-2)).kind === "object") continue;
          const value = evaluate(last);
          if (value.kind !== "object") check({ value, origin: last }, "timeout", max);
        }
      },
    };
  },
);
