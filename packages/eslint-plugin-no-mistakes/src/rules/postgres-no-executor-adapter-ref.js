"use strict";

const { childNodes, resolveVariable } = require("./postgres-query-text");
const { memberPropertyName } = require("./postgres-executor");
const { isScopedMember } = require("./postgres-scoped-executors");
const {
  TS_ASSERTIONS,
  constInitializer,
  variableIsImportedExecutor,
  variableIsScopedExecutor,
} = require("./postgres-no-executor-adapter-binding");
const {
  destructuredExecutor,
  isProvenQueryMember,
  namespaceExecutor,
} = require("./postgres-no-executor-adapter-member");
const { executorCallShape } = require("./postgres-no-executor-adapter-call-shape");

const NESTED_THIS_BOUNDARIES = new Set([
  "ClassDeclaration",
  "ClassExpression",
  "FunctionDeclaration",
  "FunctionExpression",
]);

function withCallShape(node, ref, context) {
  if (!ref) return null;
  return { ...ref, ...executorCallShape(node, context) };
}

function executorRef(node, context, bindings, options) {
  if (node.type === "ChainExpression") return null;
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression") return null;
  if (current.type === "Identifier") {
    const variable = resolveVariable(current, context);
    // Only an imported executor gets an alias hint. Analysis judges a const alias
    // of a scoped factory or typed parameter, but this rule does not suggest it.
    if (variableIsImportedExecutor(variable, options)) {
      return withCallShape(
        current,
        { name: current.name, kind: "identifier", aliasable: true },
        context,
      );
    }
    if (variableIsScopedExecutor(variable, bindings.scoped ?? [], current)) {
      return withCallShape(
        current,
        { name: current.name, kind: "identifier", aliasable: false },
        context,
      );
    }
    return withCallShape(
      current,
      destructuredExecutor(current, context, bindings, options),
      context,
    );
  }
  if (
    current.type === "MemberExpression" &&
    !current.optional &&
    current.object.type === "Identifier" &&
    isScopedMember(
      bindings.scoped,
      current.object.name,
      memberPropertyName(current),
      resolveVariable(current.object, context),
    )
  ) {
    return withCallShape(
      current,
      { name: context.sourceCode.getText(current), kind: "member", aliasable: false },
      context,
    );
  }
  if (isProvenQueryMember(current, context, bindings, options)) {
    return withCallShape(
      current,
      { name: context.sourceCode.getText(current), kind: "member", aliasable: false },
      context,
    );
  }
  return withCallShape(current, namespaceExecutor(current, context, options), context);
}

function executorLeaves(node, context, bindings, options) {
  if (node.type === "ChainExpression") return null;
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression") return null;
  if (current.type === "ConditionalExpression") {
    const consequent = executorLeaves(current.consequent, context, bindings, options);
    const alternate = executorLeaves(current.alternate, context, bindings, options);
    if (!consequent || !alternate) return null;
    return [...consequent, ...alternate];
  }
  const ref = executorRef(current, context, bindings, options);
  return ref ? [ref] : null;
}

function containsThis(root) {
  const stack = childNodes(root);
  while (stack.length) {
    const node = stack.pop();
    if (node.type === "ThisExpression") return true;
    if (NESTED_THIS_BOUNDARIES.has(node.type)) continue;
    stack.push(...childNodes(node));
  }
  return false;
}

function parameterShape(params) {
  const names = [];
  let rest = null;
  let inexact = false;
  for (const param of params) {
    if (param.type === "Identifier") {
      names.push(param.name);
      continue;
    }
    if (param.type === "AssignmentPattern" && param.left?.type === "Identifier") {
      names.push(param.left.name);
      inexact = true;
      continue;
    }
    if (param.type === "RestElement" && param.argument?.type === "Identifier") {
      rest = param.argument.name;
      inexact = true;
      continue;
    }
    return null;
  }
  return { names, rest, inexact };
}

function unwrapArgument(node) {
  let assertion = false;
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) {
    assertion = true;
    current = current.expression;
  }
  if (current?.type === "ChainExpression") return { node: null, assertion };
  return { node: current, assertion };
}

function matchForward(params, args) {
  const shape = parameterShape(params);
  if (!shape) return "unrelated";
  const parts = args.map((arg) => {
    if (arg.type === "SpreadElement") {
      const inner = unwrapArgument(arg.argument);
      return { spread: true, node: inner.node, assertion: inner.assertion };
    }
    const inner = unwrapArgument(arg);
    return { spread: false, node: inner.node, assertion: inner.assertion };
  });
  const known = new Set(shape.rest ? [...shape.names, shape.rest] : shape.names);
  const forwarded = parts.some(
    (part) => part.node?.type === "Identifier" && known.has(part.node.name),
  );
  // `() => read()` matches no parameter, so it is not an exact forwarder.
  if (!forwarded) return "unrelated";
  const positional =
    parts.length === shape.names.length &&
    parts.every(
      (part, index) =>
        !part.spread && part.node?.type === "Identifier" && part.node.name === shape.names[index],
    );
  if (!positional) return "changed";
  if (shape.inexact) return "changed";
  if (parts.some((part) => part.assertion)) return "assertion";
  return "exact";
}

module.exports = {
  TS_ASSERTIONS,
  constInitializer,
  containsThis,
  executorLeaves,
  matchForward,
  unwrapArgument,
  variableIsImportedExecutor,
  variableIsScopedExecutor,
};
