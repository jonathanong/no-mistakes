"use strict";

const { stableFunction } = require("./postgres-no-executor-adapter-binding");
const { classifyFunction } = require("./postgres-no-executor-adapter-forward");
const { boundExecutor, logicalPassed } = require("./postgres-no-executor-adapter-logic");
const {
  TS_ASSERTIONS,
  constInitializer,
  executorLeaves,
} = require("./postgres-no-executor-adapter-ref");

function unwrapPassed(node) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression") return null;
  return current;
}

function passedDirect(names, leaves, conditionalText) {
  const calls = names.map((name) => `${name}(sql\`…\`)`);
  const identifiers = leaves.length > 0 && leaves.every((leaf) => leaf.kind === "identifier");
  if (conditionalText && identifiers) calls.push(`(${conditionalText})(sql\`…\`)`);
  return calls.join(" or ");
}

function passedAlias(names, leaves) {
  if (names.length !== 1 || leaves.length === 0) return "";
  if (!leaves.every((leaf) => leaf.aliasable)) return "";
  return `, or \`const exec = ${names[0]}; exec(sql\`…\`)\``;
}

// Proven branches only. An unknown branch is omitted: naming it, or the whole
// conditional, would claim it is an executor. Deleting it is not a rewrite.
function combineConditional(consequent, alternate) {
  const known = [consequent, alternate].filter((part) => part);
  if (known.length === 0) return null;
  const names = [
    ...new Set(
      known.flatMap((part) =>
        part.adapter ? part.adapter.names : part.leaves.map((leaf) => leaf.name),
      ),
    ),
  ];
  const leaves = known.flatMap((part) => (part.adapter ? part.adapter.leaves : part.leaves));
  const definite = Boolean(consequent?.definite && alternate?.definite);
  if (!known.some((part) => part.adapter)) return { leaves, conditionalText: null, definite };
  return {
    adapter: { names, reason: "conditional selection", leaves, conditionalText: null },
    definite,
  };
}

// An executor value, a const or stable function that carries one, bind, or
// && / || / ??. Fresh objects and constructor arguments are visited by the rule.
function resolvePassed(node, context, bindings, options, seen) {
  const current = unwrapPassed(node);
  if (!current) return null;
  if (
    current.type === "Identifier" ||
    current.type === "MemberExpression" ||
    current.type === "ConditionalExpression"
  ) {
    const leaves = executorLeaves(current, context, bindings, options);
    if (leaves) {
      return {
        leaves,
        conditionalText:
          current.type === "ConditionalExpression" ? context.sourceCode.getText(current) : null,
        definite: true,
      };
    }
  }
  if (current.type === "ConditionalExpression") {
    // A const alias inside a branch is not the judged `(flag ? read : write)(sql)` callee.
    return combineConditional(
      resolvePassed(current.consequent, context, bindings, options, seen),
      resolvePassed(current.alternate, context, bindings, options, seen),
    );
  }
  if (current.type === "LogicalExpression") {
    return logicalPassed(
      current,
      (part) => resolvePassed(part, context, bindings, options, seen),
      context,
      combineConditional,
    );
  }
  if (current.type === "CallExpression") {
    return boundExecutor(current, (part) => resolvePassed(part, context, bindings, options, seen));
  }
  if (current.type === "Identifier") {
    const init = constInitializer(current, context, seen);
    if (init) return resolvePassed(init, context, bindings, options, seen);
    const fn = stableFunction(current, context);
    if (!fn) return null;
    const adapter = classifyFunction(fn, context, bindings, options);
    return adapter ? { adapter, definite: true } : null;
  }
  if (current.type === "ArrowFunctionExpression" || current.type === "FunctionExpression") {
    const adapter = classifyFunction(current, context, bindings, options);
    return adapter ? { adapter, definite: true } : null;
  }
  return null;
}

module.exports = {
  passedAlias,
  passedDirect,
  resolvePassed,
};
