"use strict";

const { childNodes, resolveVariable } = require("./postgres-query-text");

// Shared by ref and member. Neither of those modules owns this file, so
// executorRef can import the member check without a require cycle.
const TS_ASSERTIONS = new Set([
  "TSAsExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
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

function isDirectEval(node) {
  if (node?.type !== "CallExpression") return false;
  const callee = node.callee;
  return callee?.type === "Identifier" && callee.name === "eval";
}

function containsDirectEval(root) {
  const stack = childNodes(root);
  while (stack.length) {
    const node = stack.pop();
    if (isDirectEval(node)) return true;
    stack.push(...childNodes(node));
  }
  return false;
}

// A function declaration is stable only when nothing assigns it and no direct
// eval in its scope can. A function expression's inner name is a different binding.
function stableFunction(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  if (def.type !== "FunctionName" || def.node?.type !== "FunctionDeclaration") return null;
  if (variable.references.some((reference) => reference.isWrite())) return null;
  if (containsDirectEval(variable.scope.block)) return null;
  return def.node;
}

module.exports = {
  TS_ASSERTIONS,
  constInitializer,
  stableFunction,
  variableIsImportedExecutor,
  variableIsScopedExecutor,
};
