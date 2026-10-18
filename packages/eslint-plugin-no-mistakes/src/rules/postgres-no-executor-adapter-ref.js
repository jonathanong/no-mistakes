"use strict";

const { childNodes, resolveVariable } = require("./postgres-query-text");

const TS_ASSERTIONS = new Set([
  "TSAsExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);
const NESTED_THIS_BOUNDARIES = new Set([
  "ClassDeclaration",
  "ClassExpression",
  "FunctionDeclaration",
  "FunctionExpression",
]);

function importedName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function variableIsImportedExecutor(variable, options) {
  if (!variable?.defs?.length) return false;
  return variable.defs.every((def) => {
    if (def.type !== "ImportBinding" || def.node?.type !== "ImportSpecifier") return false;
    const declaration = def.parent;
    if (
      declaration?.type !== "ImportDeclaration" ||
      declaration.importKind === "type" ||
      def.node.importKind === "type"
    ) {
      return false;
    }
    const imported = importedName(def.node);
    if (!options.executorNames.includes(imported)) return false;
    return !options.importSpecifier || declaration.source?.value === options.importSpecifier;
  });
}

function variableIsScopedExecutor(variable, scoped, node) {
  if (!variable?.defs || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  return scoped.some((entry) => {
    if (
      entry.name !== node.name ||
      node.range[0] < entry.range[0] ||
      node.range[0] >= entry.range[1]
    ) {
      return false;
    }
    if (def.type === "Parameter") {
      const block = variable.scope?.block;
      return block?.range?.[0] === entry.range[0] && block?.range?.[1] === entry.range[1];
    }
    return def.type === "Variable" && def.node?.range?.[0] === entry.range[0];
  });
}

function executorRef(node, context, bindings, options) {
  if (node.type === "ChainExpression") return null;
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression") return null;
  if (current.type === "Identifier") {
    const variable = resolveVariable(current, context);
    // A const alias of an imported executor is judged. A const alias of a
    // scoped factory or typed parameter is not, so only imports are aliasable.
    if (variableIsImportedExecutor(variable, options)) {
      return { name: current.name, kind: "identifier", aliasable: true };
    }
    if (variableIsScopedExecutor(variable, bindings.scoped ?? [], current)) {
      return { name: current.name, kind: "identifier", aliasable: false };
    }
    return null;
  }
  if (
    require("./postgres-no-executor-adapter-member").isProvenQueryMember(
      current,
      context,
      bindings,
      options,
    )
  ) {
    return { name: context.sourceCode.getText(current), kind: "member", aliasable: false };
  }
  return null;
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

// One const initializer, or null when the binding can change or is not a simple name.
function constInitializer(identifier, context, seen) {
  const variable = resolveVariable(identifier, context);
  const def = variable?.defs?.length === 1 ? variable.defs[0] : null;
  // eslint-scope puts `const` on the VariableDeclaration parent, not on the def.
  if (!def || def.type !== "Variable" || def.parent.kind !== "const" || seen.has(def.node)) {
    return null;
  }
  if (def.node?.id?.type !== "Identifier" || !def.node.init) return null;
  seen.add(def.node);
  return def.node.init;
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
