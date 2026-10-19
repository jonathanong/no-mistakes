"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
} = require("./postgres-executor");
const { TS_ASSERTIONS, executorLeaves } = require("./postgres-no-executor-adapter-ref");
const {
  aliasHint,
  classifyFunction,
  directCall,
  judgedDirect,
} = require("./postgres-no-executor-adapter-forward");
const {
  passedAlias,
  passedDirect,
  resolvePassed,
} = require("./postgres-no-executor-adapter-passed");

const RULE_ID = "postgres-no-executor-adapter";

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow executor forwarding adapters that hide SQL from analysis",
      recommended: false,
    },
    schema: [executorOptionSchema()],
    messages: {
      forward:
        "This callback only forwards to {{executor}}. Call that executor on the SQL text (`{{direct}}`{{alias}}). Passing the function into a helper (`{{rejected}}`) does not connect the statement to the executor, so analysis cannot see it.",
      blocked:
        "This adapter hides {{executor}} from SQL analysis ({{reason}}). Call the executor on the SQL text (`{{direct}}`{{alias}}). `{{rejected}}` is not a judged rewrite: analysis does not follow an executor passed as a value. Write the direct call explicitly. Analysis cannot check SQL that only reaches the executor through this adapter.",
      passed:
        "This call passes {{executor}} into a helper. Name the executor inside the helper (`{{direct}}`{{alias}}) instead of passing it. Analysis does not follow an executor passed as a value, so this call is not a judged rewrite. Write the SQL as an argument of that executor. Analysis cannot check SQL that only reaches the executor through a passed function.",
    },
  },
  (context) => {
    const options = executorOptionDefaults(context.options?.[0] ?? {});
    const suppressed = ruleSuppression(context, RULE_ID);
    const source = context.sourceCode;
    let bindings = { scoped: [], queryMembers: false };

    function reportAdapter(node, adapter, call, index) {
      if (suppressed(node)) return;
      const names = adapter.names;
      const leaves = adapter.leaves ?? [];
      const rejected = names.map((name) => directCall(source, call, index, name)).join(" or ");
      context.report({
        node,
        messageId: adapter.reason ? "blocked" : "forward",
        data: {
          executor: names.join(" or "),
          direct: judgedDirect(names, leaves, adapter.conditionalText),
          alias: aliasHint(names, leaves),
          rejected,
          reason: adapter.reason ?? "the selector contract changed",
        },
      });
    }

    function reportPassed(node, passed) {
      if (suppressed(node)) return;
      const names = [...new Set(passed.leaves.map((leaf) => leaf.name))];
      context.report({
        node,
        messageId: "passed",
        data: {
          executor: names.join(" or "),
          direct: passedDirect(names, passed.leaves, passed.conditionalText),
          alias: passedAlias(names, passed.leaves),
        },
      });
    }

    function visitArgument(argument, call, index) {
      if (!argument || argument.type === "SpreadElement") return;
      let expr = argument;
      let wrapped = false;
      while (expr && TS_ASSERTIONS.has(expr.type)) {
        wrapped = true;
        expr = expr.expression;
      }
      if (!expr || expr.type === "ChainExpression") return;
      if (expr.type === "ConditionalExpression") {
        const leaves = executorLeaves(expr, context, bindings, options);
        // Every branch is an executor, so the conditional callee itself is judged.
        // A partial conditional falls through and names only the proven branches.
        if (leaves) {
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
          );
          return;
        }
      }
      if (expr.type === "ArrowFunctionExpression" || expr.type === "FunctionExpression") {
        const adapter = classifyFunction(expr, context, bindings, options);
        if (!adapter) return;
        if (wrapped && !adapter.reason) adapter.reason = "type assertion";
        reportAdapter(argument, adapter, call, index);
        return;
      }
      const passed = resolvePassed(expr, context, bindings, options, new Set());
      if (!passed) return;
      if (passed.adapter) {
        if (wrapped && !passed.adapter.reason) passed.adapter.reason = "type assertion";
        reportAdapter(argument, passed.adapter, call, index);
        return;
      }
      reportPassed(argument, passed);
    }

    return {
      Program(node) {
        bindings = executorBindings(node, options, context.filename);
      },
      CallExpression(node) {
        node.arguments.forEach((argument, index) => visitArgument(argument, node, index));
      },
    };
  },
);
