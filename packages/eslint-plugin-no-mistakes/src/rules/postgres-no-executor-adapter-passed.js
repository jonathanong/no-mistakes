"use strict";

const { classifyFunction } = require("./postgres-no-executor-adapter-forward");
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

// An executor value, a const of those values, or a const adapter. Constructors
// and object properties are not followed.
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
      };
    }
  }
  if (current.type === "ConditionalExpression") {
    const consequent = resolvePassed(current.consequent, context, bindings, options, seen);
    const alternate = resolvePassed(current.alternate, context, bindings, options, seen);
    if (!consequent?.leaves || !alternate?.leaves) return null;
    // A const alias inside a branch is not the judged `(flag ? read : write)(sql)` callee.
    return { leaves: [...consequent.leaves, ...alternate.leaves], conditionalText: null };
  }
  if (current.type === "Identifier") {
    const init = constInitializer(current, context, seen);
    if (!init) return null;
    return resolvePassed(init, context, bindings, options, seen);
  }
  if (current.type === "ArrowFunctionExpression" || current.type === "FunctionExpression") {
    const adapter = classifyFunction(current, context, bindings, options);
    return adapter ? { adapter } : null;
  }
  return null;
}

module.exports = {
  passedAlias,
  passedDirect,
  resolvePassed,
};
