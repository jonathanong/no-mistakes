"use strict";

const TS_WRAPPERS = new Set([
  "ChainExpression",
  "TSAsExpression",
  "TSInstantiationExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
]);

function unwrapTs(node) {
  while (node && TS_WRAPPERS.has(node.type)) {
    node = node.expression;
  }
  return node;
}

function childNodes(node) {
  const children = [];
  if (!node || typeof node !== "object") return children;
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        if (item?.type) children.push(item);
      }
    } else if (value?.type) {
      children.push(value);
    }
  }
  return children;
}

function quasiText(quasi) {
  return quasi?.value?.cooked ?? quasi?.value?.raw ?? "";
}

function templateSqlText(template) {
  let out = "";
  const quasis = template?.quasis ?? [];
  for (let index = 0; index < quasis.length; index += 1) {
    if (index > 0) out += `sql_placeholder_${index}`;
    out += quasiText(quasis[index]);
  }
  return out;
}

function sqlText(node) {
  node = unwrapTs(node);
  if (!node) return null;
  if (node.type === "Literal" && typeof node.value === "string") return node.value;
  if (node.type === "TemplateLiteral") return templateSqlText(node);
  if (node.type === "TaggedTemplateExpression") return templateSqlText(node.quasi);
  return null;
}

function variableFromScope(scope, name) {
  const get = scope?.set?.get;
  if (typeof get === "function") return get.call(scope.set, name) ?? null;
  return scope?.variables?.find((item) => item.name === name) ?? null;
}

function resolveVariable(node, context) {
  if (node?.type !== "Identifier" || !context?.sourceCode?.getScope) return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = variableFromScope(scope, node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function queryTextFromScope(ident, context) {
  const variable = resolveVariable(ident, context);
  if (!variable) return null;
  if (variable.defs.some((def) => def.type === "Parameter" || def.type === "CatchClause")) {
    return null;
  }
  for (const def of variable.defs) {
    if (def.type !== "Variable" || def.node?.id?.type !== "Identifier") continue;
    const text = sqlText(def.node.init);
    if (text != null) return text;
  }
  return null;
}

function executedQueryText(node, bindings, context) {
  const text = sqlText(node);
  if (text != null) return text;
  node = unwrapTs(node);
  if (node?.type !== "Identifier") return null;
  if (context) {
    const scoped = queryTextFromScope(node, context);
    if (scoped != null) return scoped;
    const variable = resolveVariable(node, context);
    if (variable) return null;
  }
  return bindings instanceof Map ? (bindings.get(node.name) ?? null) : null;
}

function sqlStatementBindings(root) {
  const bindings = new Map();
  function visit(node) {
    if (node.type === "VariableDeclarator" && node.id?.type === "Identifier") {
      const text = sqlText(node.init);
      if (text != null) bindings.set(node.id.name, text);
    }
    for (const child of childNodes(node)) visit(child);
  }
  if (root?.type) visit(root);
  return bindings;
}

module.exports = {
  childNodes,
  executedQueryText,
  quasiText,
  resolveVariable,
  sqlStatementBindings,
  sqlText,
  templateSqlText,
  unwrapTs,
};
