"use strict";
const { rule, options } = require("../helpers");
const { isFunction } = require("./async-ast");
const { ruleSuppression } = require("../rule-suppression");
const { configCall, overrideCall } = require("./vitest-timeout-bindings");
const { createEvaluator } = require("./vitest-timeout-values");
module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "cap Vitest project defaults and test/hook timeout overrides",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          defaultMax: { type: "number", minimum: 0, exclusiveMinimum: true },
          overrideMax: { type: "number", minimum: 0, exclusiveMinimum: true },
          unknownValues: { enum: ["ignore", "finding"] },
        },
        additionalProperties: false,
      },
    ],
    messages: {
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
    function check(entry, name, max, origin) {
      const node = origin || entry.origin;
      if (suppressed(node)) return;
      if (entry.value.kind === "number") {
        if (entry.value.value > max)
          context.report({ node, messageId: "timeout", data: { name, max } });
      } else if (option.unknownValues === "finding")
        context.report({ node, messageId: "unknown", data: { name, max } });
    }
    function properties(value, names, max, origin) {
      if (value.kind !== "object") return;
      for (const name of names) {
        const entry = value.properties.get(name);
        if (entry) check(entry, name, max, origin);
      }
    }
    function defaults(value, project) {
      if (value.kind !== "object") return;
      const test = value.properties.get("test")?.value;
      if (!test) return;
      properties(test, ["testTimeout", "hookTimeout"], option.defaultMax ?? 5000, project);
      const projects = test.kind === "object" ? test.properties.get("projects")?.value : null;
      if (projects?.kind === "array")
        for (const entry of projects.entries) defaults(entry.value, entry.origin);
    }
    return {
      ExportDefaultDeclaration(node) {
        exports.push(node.declaration);
      },
      CallExpression(node) {
        if (configCall(node, context)) configs.push(node);
        const kind = overrideCall(node, context);
        if (kind) overrides.push({ node, kind });
      },
      "Program:exit"() {
        // Evaluate exports and outer merges first so overwritten fragment values never report.
        for (const node of exports) {
          const count = consumed.size;
          const value = evaluate(node);
          if (consumed.size > count) defaults(value);
        }
        const roots = configs.filter(
          (node) =>
            !configs.some(
              (other) =>
                other !== node && other.range[0] < node.range[0] && other.range[1] > node.range[1],
            ),
        );
        for (const node of roots.toReversed()) if (!consumed.has(node)) defaults(evaluate(node));
        for (const { node, kind } of overrides) {
          const max = option.overrideMax ?? 30000;
          if (kind === "runtime") {
            properties(
              evaluate(node.arguments[0]),
              ["testTimeout", "hookTimeout"],
              max,
              node.arguments[0]?.type === "Identifier" ? node.arguments[0] : undefined,
            );
            continue;
          }
          const args = kind === "hook" ? node.arguments.slice(1) : node.arguments.slice(1, 3);
          for (const argument of args) {
            const value = evaluate(argument);
            if (value.kind === "object")
              properties(
                value,
                ["timeout"],
                max,
                argument.type === "Identifier" ? argument : undefined,
              );
          }
          const last = node.arguments.at(-1);
          const minimum = kind === "hook" ? 2 : 3;
          if (!last || node.arguments.length < minimum || isFunction(last)) continue;
          // An options object followed by a callback is not a trailing timeout.
          if (evaluate(node.arguments.at(-2)).kind === "object") continue;
          const value = evaluate(last);
          if (value.kind !== "object") check({ value, origin: last }, "timeout", max);
        }
      },
    };
  },
);
