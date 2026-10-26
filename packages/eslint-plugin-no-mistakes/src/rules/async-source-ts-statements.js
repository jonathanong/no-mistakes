"use strict";
const { assignmentTarget } = require("./async-source-ts-expressions");

function statement(node, api) {
  const { ts, convert: c, make: m, list } = api;
  if (ts.isImportDeclaration(node)) {
    if (node.attributes || node.assertClause) throw new Error("Unsupported import attributes");
    const clause = node.importClause;
    const specifiers = [];
    if (clause?.name)
      specifiers.push(m(clause.name, "ImportDefaultSpecifier", { local: c(clause.name) }));
    const bindings = clause?.namedBindings;
    if (bindings && ts.isNamespaceImport(bindings))
      specifiers.push(m(bindings, "ImportNamespaceSpecifier", { local: c(bindings.name) }));
    else if (bindings)
      for (const item of bindings.elements)
        specifiers.push(
          m(item, "ImportSpecifier", {
            local: c(item.name),
            imported: c(item.propertyName || item.name),
            importKind: item.isTypeOnly ? "type" : "value",
          }),
        );
    return m(node, "ImportDeclaration", {
      source: c(node.moduleSpecifier),
      specifiers,
      importKind: clause?.isTypeOnly ? "type" : "value",
    });
  }
  if (ts.isExportDeclaration(node)) {
    if (node.attributes || node.assertClause) throw new Error("Unsupported export attributes");
    if (!node.exportClause || ts.isNamespaceExport(node.exportClause))
      return m(node, "ExportAllDeclaration", {
        source: c(node.moduleSpecifier),
        exported: c(node.exportClause?.name),
        exportKind: node.isTypeOnly ? "type" : "value",
      });
    return m(node, "ExportNamedDeclaration", {
      source: c(node.moduleSpecifier),
      declaration: null,
      exportKind: node.isTypeOnly ? "type" : "value",
      specifiers: node.exportClause.elements.map((item) =>
        m(item, "ExportSpecifier", {
          local: c(item.propertyName || item.name),
          exported: c(item.name),
          exportKind: item.isTypeOnly ? "type" : "value",
        }),
      ),
    });
  }
  if (ts.isBlock(node)) return m(node, "BlockStatement", { body: list(node.statements) });
  if (ts.isExpressionStatement(node))
    return m(node, "ExpressionStatement", { expression: c(node.expression) });
  if (ts.isReturnStatement(node) || ts.isThrowStatement(node))
    return m(node, ts.isReturnStatement(node) ? "ReturnStatement" : "ThrowStatement", {
      argument: c(node.expression),
    });
  if (ts.isIfStatement(node))
    return m(node, "IfStatement", {
      test: c(node.expression),
      consequent: c(node.thenStatement),
      alternate: c(node.elseStatement),
    });
  if (ts.isForOfStatement(node) || ts.isForInStatement(node))
    return m(node, ts.isForOfStatement(node) ? "ForOfStatement" : "ForInStatement", {
      left: assignmentTarget(node.initializer, api),
      right: c(node.expression),
      body: c(node.statement),
      await: Boolean(node.awaitModifier),
    });
  if (ts.isForStatement(node))
    return m(node, "ForStatement", {
      init: c(node.initializer),
      test: c(node.condition),
      update: c(node.incrementor),
      body: c(node.statement),
    });
  if (ts.isWhileStatement(node) || ts.isDoStatement(node))
    return m(node, ts.isDoStatement(node) ? "DoWhileStatement" : "WhileStatement", {
      test: c(node.expression),
      body: c(node.statement),
    });
  if (ts.isTryStatement(node))
    return m(node, "TryStatement", {
      block: c(node.tryBlock),
      handler: c(node.catchClause),
      finalizer: c(node.finallyBlock),
    });
  if (ts.isCatchClause(node))
    return m(node, "CatchClause", {
      param: c(node.variableDeclaration?.name),
      body: c(node.block),
    });
  if (ts.isEmptyStatement(node)) return m(node, "EmptyStatement");
  if (ts.isBreakStatement(node) || ts.isContinueStatement(node)) {
    if (node.label) throw new Error("Unsupported labelled statement");
    return m(node, ts.isBreakStatement(node) ? "BreakStatement" : "ContinueStatement");
  }
  return null;
}

module.exports = { statement };
