"use strict";

const { childNodes, resolveVariable } = require("./postgres-query-text");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const {
  localAliasField,
  localField,
  projectedScalar,
  tsAliasField,
  tsField,
  tsObjectAliased,
} = require("./postgres-no-executor-adapter-sql-projection");

function localHoleSafe(expression, call, fn, context, executorVariable, allowedProperty) {
  const parameters = fn.params.map((param) => {
    const target = param.type === "AssignmentPattern" ? param.left : param;
    return target.type === "Identifier" ? resolveVariable(target, context) : null;
  });
  const fields = new Map();
  fn.params.forEach((param, index) => {
    const target = param.type === "AssignmentPattern" ? param.left : param;
    if (target.type !== "ObjectPattern") return;
    for (const entry of target.properties) {
      if (entry.type !== "Property" || entry.value.type !== "Identifier") continue;
      const variable = resolveVariable(entry.value, context);
      const field = localField(target, variable, context);
      if (variable && field !== null) fields.set(variable, { index, field });
    }
  });
  const requirements = [];
  const seen = new Map();
  function visit(node, field = null) {
    if (node.type === "MemberExpression" && !node.computed) {
      if (node.property.type !== "Identifier") {
        requirements.push(null);
        return;
      }
      if (node.parent?.type === "CallExpression" && node.parent.callee === node)
        return visit(node.object, field);
      if (field !== null) requirements.push(null);
      return visit(node.object, node.property.name);
    }
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      if (!variable) return;
      const visited = seen.get(variable) ?? new Set();
      if (visited.has(field)) return;
      visited.add(field);
      seen.set(variable, visited);
      const index = parameters.indexOf(variable);
      if (index >= 0) {
        requirements.push({ index, field });
        if (
          variable.references.some((ref) => ref.isWrite() && !ref.init) ||
          (field !== null &&
            !(call.arguments[index] && projectedScalar(call.arguments[index], null, context)) &&
            objectMutation(variable, executorVariable, allowedProperty))
        )
          requirements.push(null);
      } else if (fields.has(variable)) {
        const binding = fields.get(variable);
        requirements.push(field === null ? binding : null);
        if (variable.references.some((ref) => ref.isWrite() && !ref.init)) requirements.push(null);
      } else if (variable.defs.some((def) => def.type === "Parameter")) requirements.push(null);
      else
        for (const def of variable.defs) {
          if (def.type === "Variable") {
            if (variable.references.some((ref) => ref.isWrite() && !ref.init))
              requirements.push(null);
            const alias = localAliasField(def, variable, context);
            if (alias) {
              if (field !== null) requirements.push(null);
              visit(alias.initializer, alias.field);
            } else if (def.node.init) visit(def.node.init, field);
            else if (def.node.id.type !== "Identifier") requirements.push(null);
          }
        }
      return;
    }
    for (const child of childNodes(node)) visit(child, field);
  }
  visit(expression);
  return requirements.every((requirement) => {
    if (!requirement) return false;
    const { index, field } = requirement;
    const fallback = fn.params[index]?.type === "AssignmentPattern" ? fn.params[index].right : null;
    const defaultScalar = fallback ? projectedScalar(fallback, field, context) : true;
    return call.arguments[index]
      ? projectedScalar(call.arguments[index], field, context, defaultScalar)
      : Boolean(fallback && defaultScalar);
  });
}

function tsScalarDefault(node, ts) {
  return Boolean(
    node &&
    (ts.isStringLiteral(node) ||
      ts.isNumericLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node) ||
      [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword, ts.SyntaxKind.NullKeyword].includes(
        node.kind,
      )),
  );
}

function tsHoleSafe(
  expression,
  call,
  declaration,
  checker,
  ts,
  context,
  facts,
  program,
  executorSymbol,
  allowedProperty,
) {
  const parameters = declaration.parameters.map((param) =>
    ts.isIdentifier(param.name) ? checker.getSymbolAtLocation(param.name) : null,
  );
  const fields = new Map();
  declaration.parameters.forEach((param, index) => {
    if (!ts.isObjectBindingPattern(param.name)) return;
    for (const entry of param.name.elements) {
      if (!ts.isIdentifier(entry.name)) continue;
      const symbol = checker.getSymbolAtLocation(entry.name);
      const field = tsField(param.name, symbol, checker, ts);
      if (symbol && field !== null) fields.set(symbol, { index, field });
    }
  });
  const requirements = [];
  const seen = new Map();
  function visit(node, field = null) {
    if (ts.isPropertyAccessExpression(node)) {
      if (ts.isCallExpression(node.parent) && node.parent.expression === node)
        return visit(node.expression, field);
      if (field !== null) requirements.push(null);
      return visit(node.expression, node.name.text);
    }
    if (ts.isIdentifier(node)) {
      const symbol = checker.getSymbolAtLocation(node);
      if (!symbol) return;
      const visited = seen.get(symbol) ?? new Set();
      if (visited.has(field)) return;
      visited.add(field);
      seen.set(symbol, visited);
      const index = parameters.indexOf(symbol);
      if (index >= 0) {
        requirements.push({ index, field });
        if (
          sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol) ||
          (field !== null &&
            !(call.arguments[index] && projectedScalar(call.arguments[index], null, context)) &&
            tsObjectAliased(
              declaration,
              symbol,
              checker,
              ts,
              facts,
              executorSymbol,
              allowedProperty,
            ))
        )
          requirements.push(null);
      } else if (fields.has(symbol)) {
        const binding = fields.get(symbol);
        requirements.push(field === null ? binding : null);
        if (sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol))
          requirements.push(null);
      } else {
        if (sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol))
          requirements.push(null);
        for (const source of symbol.declarations ?? []) {
          if (ts.isParameter(source)) requirements.push(null);
          if (ts.isBindingElement(source)) {
            const alias = tsAliasField(source, symbol, checker, ts);
            if (alias) {
              if (field !== null) requirements.push(null);
              visit(alias.initializer, alias.field);
            } else requirements.push(null);
          }
          if (ts.isVariableDeclaration(source) && source.initializer)
            visit(source.initializer, field);
        }
      }
      return;
    }
    ts.forEachChild(node, (child) => visit(child, field));
  }
  visit(expression);
  return requirements.every((requirement) => {
    if (!requirement) return false;
    const { index, field } = requirement;
    const fallback = declaration.parameters[index]?.initializer;
    const defaultScalar = fallback ? field === null && tsScalarDefault(fallback, ts) : true;
    return call.arguments[index]
      ? projectedScalar(call.arguments[index], field, context, defaultScalar)
      : Boolean(fallback && defaultScalar);
  });
}
module.exports = { localHoleSafe, tsHoleSafe };
