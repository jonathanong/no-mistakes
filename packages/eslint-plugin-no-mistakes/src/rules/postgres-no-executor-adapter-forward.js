"use strict";

const { propertyText } = require("./postgres-no-executor-adapter-logic");
const {
  TS_ASSERTIONS,
  containsThis,
  executorLeaves,
  matchForward,
  unwrapArgument,
} = require("./postgres-no-executor-adapter-ref");

// `.call` / `.apply` shift arguments only when the object is already an executor.
// A static key (`read["call"]`, `read[\`apply\`]`) is that method. A member
// named `call` whose object is not an executor stays the member.
function forwardedMember(call, callee, context, bindings, options) {
  const name = propertyText(callee);
  if (name !== "call" && name !== "apply") {
    return null;
  }
  const leaves = executorLeaves(callee.object, context, bindings, options);
  if (!leaves) {
    return null;
  }
  const args = name === "call" ? call.arguments.slice(1) : applyArguments(call);
  const object = callee.object;
  return {
    leaves,
    args,
    receiver: true,
    conditionalText:
      object.type === "ConditionalExpression" ? context.sourceCode.getText(object) : null,
  };
}

function calleeForward(call, context, bindings, options) {
  const callee = call.callee;
  if (callee?.type === "MemberExpression") {
    const forwarded = forwardedMember(call, callee, context, bindings, options);
    if (forwarded) {
      return forwarded;
    }
  }
  const leaves = executorLeaves(callee, context, bindings, options);
  if (!leaves) return null;
  const conditional = leaves.length > 1 || callee.type === "ConditionalExpression";
  return {
    leaves,
    args: call.arguments,
    receiver: !conditional && leaves.some((leaf) => leaf.kind === "member"),
    conditional,
    conditionalText:
      callee.type === "ConditionalExpression" ? context.sourceCode.getText(callee) : null,
  };
}

function applyArguments(call) {
  const list = unwrapArgument(call.arguments[1]).node;
  if (!list) return [];
  if (list.type !== "ArrayExpression") return [list];
  return list.elements.filter((element) => element != null);
}

function classifyCall(call, fn, context, bindings, options, flags) {
  const forward = calleeForward(call, context, bindings, options);
  if (!forward) return null;
  const matched = matchForward(fn.params, forward.args);
  if (matched === "unrelated") return null;
  const names = [...new Set(forward.leaves.map((leaf) => leaf.name))];
  let reason = null;
  if (forward.conditional) reason = "conditional selection";
  else if (forward.receiver) reason = "receiver or this";
  else if (matched === "changed") reason = "arity or rest";
  else if (flags.sideEffects) reason = "side effects";
  else if (flags.usesThis) reason = "receiver or this";
  else if (flags.awaited || fn.async || fn.generator) reason = "async";
  else if (matched === "assertion") reason = "type assertion";
  return {
    names,
    reason,
    leaves: forward.leaves,
    conditionalText: forward.conditionalText ?? null,
  };
}

function classifyExpr(expr, fn, context, bindings, options, flags) {
  // `read?.(statement)` is the same forward as `read(statement)`.
  if (expr.type === "ChainExpression") {
    return classifyExpr(expr.expression, fn, context, bindings, options, flags);
  }
  if (TS_ASSERTIONS.has(expr.type)) {
    const inner = classifyExpr(expr.expression, fn, context, bindings, options, flags);
    if (!inner) return null;
    if (!inner.reason) inner.reason = "type assertion";
    return inner;
  }
  if (expr.type === "ConditionalExpression") {
    const branches = [expr.consequent, expr.alternate]
      .map((branch) => classifyExpr(branch, fn, context, bindings, options, flags))
      .filter((branch) => branch);
    // An unknown branch is omitted. Naming it, or `(flag ? read : other)(sql)`, would
    // claim it is an executor. Call branches are not an executor-valued callee.
    if (branches.length === 0) return null;
    return {
      names: [...new Set(branches.flatMap((branch) => branch.names))],
      reason: "conditional selection",
      leaves: branches.flatMap((branch) => branch.leaves),
      conditionalText: null,
    };
  }
  if (expr.type === "SequenceExpression") {
    const last = expr.expressions[expr.expressions.length - 1];
    return classifyExpr(last, fn, context, bindings, options, { ...flags, sideEffects: true });
  }
  if (expr.type === "UnaryExpression" && expr.operator === "void") {
    return classifyExpr(expr.argument, fn, context, bindings, options, {
      ...flags,
      sideEffects: true,
    });
  }
  if (expr.type === "AwaitExpression") {
    return classifyExpr(expr.argument, fn, context, bindings, options, { ...flags, awaited: true });
  }
  if (expr.type !== "CallExpression") return null;
  return classifyCall(expr, fn, context, bindings, options, flags);
}

function returnedExpression(fn) {
  const body = fn.body;
  if (!body) return null;
  if (body.type !== "BlockStatement") return { expr: body, sideEffects: false };
  if (body.body.length === 0) return null;
  const last = body.body[body.body.length - 1];
  const sideEffects = body.body.length > 1 || last.type === "ExpressionStatement";
  if (last.type === "ReturnStatement") {
    return last.argument ? { expr: last.argument, sideEffects } : null;
  }
  if (last.type === "ExpressionStatement") return { expr: last.expression, sideEffects };
  return null;
}

function classifyFunction(fn, context, bindings, options) {
  const returned = returnedExpression(fn);
  if (!returned) return null;
  return classifyExpr(returned.expr, fn, context, bindings, options, {
    sideEffects: returned.sideEffects,
    awaited: false,
    usesThis: containsThis(fn),
  });
}

function directCall(source, call, index, name, quoteNode) {
  const args = call.arguments.map((argument, argumentIndex) => {
    if (argumentIndex !== index) return source.getText(argument);
    if (!quoteNode) return name;
    const text = source.getText(argument);
    const start = quoteNode.range[0] - argument.range[0];
    const end = quoteNode.range[1] - argument.range[0];
    return text.slice(0, start) + name + text.slice(end);
  });
  const invoked = `${source.getText(call.callee)}(${args.join(", ")})`;
  return call.type === "NewExpression" ? `new ${invoked}` : invoked;
}

function judgedDirect(names, leaves, conditionalText) {
  const calls = names.map((name) => `${name}(sql)`);
  const identifiers = leaves.length > 0 && leaves.every((leaf) => leaf.kind === "identifier");
  if (conditionalText && identifiers) calls.push(`(${conditionalText})(sql)`);
  return calls.join(" or ");
}

function aliasHint(names, leaves) {
  if (names.length !== 1 || leaves.length === 0) return "";
  if (!leaves.every((leaf) => leaf.aliasable)) return "";
  return `, or \`const exec = ${names[0]}; exec(sql)\``;
}

module.exports = {
  aliasHint,
  classifyFunction,
  directCall,
  judgedDirect,
};
