"use strict";

const { nonWritingCalleesSchema } = require("./postgres-no-executor-adapter-nonwriting-config");
const { targetFactsFor } = require("./postgres-no-executor-adapter-facts");

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
} = require("./postgres-executor");
const { TS_ASSERTIONS, executorLeaves } = require("./postgres-no-executor-adapter-ref");
const { classifyFunction } = require("./postgres-no-executor-adapter-forward");
const { resolvePassed } = require("./postgres-no-executor-adapter-passed");
const { createReporters } = require("./postgres-no-executor-adapter-report");
const { libraryAdapter } = require("./postgres-no-executor-adapter-library");
const { callbackTarget } = require("./postgres-no-executor-adapter-callback");
const { acceptedTarget } = require("./postgres-no-executor-adapter-supplied");
const { directOption } = require("./postgres-no-executor-adapter-direct");
const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");
const { consumeOmittedOptional } = require("./postgres-no-executor-adapter-sql-absent");
const RULE_ID = "postgres-no-executor-adapter";

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow executor forwarding adapters that hide SQL from analysis",
      recommended: false,
    },
    schema: [
      executorOptionSchema({
        mode: { enum: ["forwarding-only", "all"] },
        nonWritingCallees: nonWritingCalleesSchema,
      }),
    ],
    messages: {
      forward:
        "This callback only forwards to {{executor}}. Call that executor on the SQL text (`{{direct}}`{{alias}}). Passing the function into a helper (`{{rejected}}`) does not connect the statement to the executor, so analysis cannot see it.",
      blocked:
        "This adapter hides {{executor}} from SQL analysis ({{reason}}). Call the executor on the SQL text (`{{direct}}`{{alias}}). `{{rejected}}` is not a judged rewrite: analysis does not follow an executor passed as a value. Write the direct call explicitly. Analysis cannot check SQL that only reaches the executor through this adapter.",
      passed:
        "This call passes {{executor}} into a helper. Name the executor inside the helper{{form}} instead of passing it. Analysis does not follow an executor passed as a value, so this call is not a judged rewrite. Write the SQL as an argument of that executor. Analysis cannot check SQL that only reaches the executor through a passed function.",
    },
  },
  (context) => {
    const options = executorOptionDefaults(context.options?.[0] ?? {});
    const mode = context.options?.[0]?.mode ?? "all";
    const suppressed = ruleSuppression(context, RULE_ID);
    const source = context.sourceCode;
    const filename = context.filename;
    const targetFacts = targetFactsFor(context);
    const { reportAdapter, reportPassed } = createReporters(context, suppressed);
    let bindings = { scoped: [], queryMembers: false };

    // `statement => write(statement, options)` reaches a helper's executor
    // parameter only when the helper's own SQL is proven literal.
    function libraryClosure(expr, wrapped, call, index, quoteNode, nested) {
      return (
        !wrapped &&
        !nested &&
        !quoteNode &&
        libraryAdapter(expr, context, options) &&
        (acceptedTarget(call, index, quoteNode, context, options, targetFacts, mode) ||
          callbackTarget(call, index, context, targetFacts)) &&
        visibleSqlTarget(call, index, quoteNode, context, options, true, targetFacts, true) === true
      );
    }

    function visitArgument(argument, call, index, quoteNode, nested = false) {
      if (!argument) return;
      if (argument.type === "SpreadElement") {
        visitArgument(argument.argument, call, index, argument, true);
        return;
      }
      let expr = argument;
      let wrapped = false;
      while (expr && (TS_ASSERTIONS.has(expr.type) || expr.type === "ChainExpression")) {
        if (TS_ASSERTIONS.has(expr.type)) wrapped = true;
        expr = expr.expression;
      }
      if (expr.type === "ObjectExpression" || expr.type === "ArrayExpression") {
        visitFresh(expr, call, index, nested || Boolean(quoteNode));
        return;
      }
      if (!nested && directOption(expr, call, index, quoteNode, context, bindings, options)) return;
      if (expr.type === "ConditionalExpression") {
        const leaves = executorLeaves(expr, context, bindings, options);
        // Every branch is an executor, so the conditional callee itself is judged.
        // A partial conditional falls through and names only the proven branches.
        if (leaves) {
          const accepted =
            !nested && acceptedTarget(call, index, quoteNode, context, options, targetFacts, mode);
          const visible =
            mode === "forwarding-only" && !nested
              ? visibleSqlTarget(call, index, quoteNode, context, options, accepted, targetFacts)
              : null;
          if (visible === true || (accepted && visible !== false)) return;
          reportAdapter(
            argument,
            {
              names: [...new Set(leaves.map((leaf) => leaf.name))],
              reason: "conditional selection",
              leaves,
              conditionalText: source.getText(expr),
            },
            call,
            index,
            quoteNode,
          );
          return;
        }
      }
      if (expr.type === "ArrowFunctionExpression" || expr.type === "FunctionExpression") {
        if (libraryClosure(expr, wrapped, call, index, quoteNode, nested)) return;
        const adapter = classifyFunction(expr, context, bindings, options);
        if (!adapter) return;
        if (wrapped && !adapter.reason) adapter.reason = "type assertion";
        reportAdapter(argument, adapter, call, index, quoteNode);
        return;
      }
      const passed = resolvePassed(expr, context, bindings, options, new Set());
      if (!passed) return;
      if (passed.adapter) {
        if (wrapped && !passed.adapter.reason) passed.adapter.reason = "type assertion";
        reportAdapter(argument, passed.adapter, call, index, quoteNode);
        return;
      }
      const accepted =
        !nested && acceptedTarget(call, index, quoteNode, context, options, targetFacts, mode);
      if (mode === "all" && !nested && accepted && passed.definite) {
        const proven = visibleSqlTarget(
          call,
          index,
          quoteNode,
          context,
          options,
          accepted,
          targetFacts,
        );
        const omitted = consumeOmittedOptional();
        if (proven === false && omitted) {
          reportPassed(argument, passed);
          return;
        }
      }
      const visible =
        mode === "forwarding-only" && !nested && passed.definite
          ? visibleSqlTarget(call, index, quoteNode, context, options, accepted, targetFacts)
          : null;
      const hidden = !passed.definite || visible === false || (!accepted && !visible);
      if (nested || (mode === "forwarding-only" ? hidden : !accepted))
        reportPassed(argument, passed);
    }

    function visitFresh(expr, call, index, nested) {
      if (expr.type === "ObjectExpression") {
        const unproven = nested || expr.properties.some((entry) => entry.type === "SpreadElement");
        for (const property of expr.properties) {
          if (property.type === "SpreadElement") {
            visitArgument(property, call, index, property, true);
            continue;
          }
          if (property.type !== "Property" || property.kind !== "init") continue;
          visitArgument(property.value, call, index, property, unproven);
        }
        return;
      }
      for (const element of expr.elements) visitArgument(element, call, index, element, nested);
    }

    // The bind result is already the reported value. Its arguments are not a
    // second pass of that same executor (`run(read.bind(read), account)`).
    function bindResultIsReported(node) {
      const parent = node.parent;
      const called = parent?.type === "CallExpression" || parent?.type === "NewExpression";
      if (!called || !parent.arguments.includes(node)) return false;
      return Boolean(resolvePassed(node, context, bindings, options, new Set()));
    }

    return {
      Program(node) {
        bindings = executorBindings(node, options, filename, context, false, mode === "all");
      },
      CallExpression(node) {
        if (bindResultIsReported(node)) return;
        node.arguments.forEach((argument, index) => visitArgument(argument, node, index));
      },
      NewExpression(node) {
        node.arguments.forEach((argument, index) => visitArgument(argument, node, index));
      },
    };
  },
);
