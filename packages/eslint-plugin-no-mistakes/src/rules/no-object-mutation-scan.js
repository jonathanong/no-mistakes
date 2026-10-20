"use strict";

const api = require("./no-object-mutation-runtime");
const { bindPattern } = require("./no-object-mutation-bind");
const { walkChildren } = require("./no-object-mutation-build");
const { escapeExport } = require("./no-object-mutation-call");
const { visitClassNode } = require("./no-object-mutation-class");
const { visitExpression } = require("./no-object-mutation-expr");
const { flushPending, registerFn, visitFunction } = require("./no-object-mutation-fn");
const { visitNamespace } = require("./no-object-mutation-namespace");
const { visitLoop } = require("./no-object-mutation-loop");
const {
  seedClasses,
  seedHoisted,
  seedImports,
  TS_EXPRESSIONS,
} = require("./no-object-mutation-syntax");
const { markEscaped } = require("./no-object-mutation-write");
const { child, rootEnv, unknown } = require("./no-object-mutation-values");

// Escapes stick once seen, including on a conditional path: a later write is
// not a straight-line local builder. Function bodies wait until the enclosing
// scope has bound its locals, then run at the first call, when the function
// escapes, or at the end of that scope. Nested functions record writes to an
// outer fresh object and decide them when that function is called or escapes.
// Module, import, and parameter writes are reported while the body is walked.
// Storing a value into the module or another function escapes it.

function scanMutations(program, context, options) {
  const state = {
    reported: new Set(),
    violations: [],
    escapeEpoch: 0,
    context,
    options,
  };
  const env = rootEnv();
  seedImports(program, env);
  seedHoisted(program.body, env, true, registerFn);
  seedClasses(program.body, env);
  for (const statement of program.body) {
    visit(state, statement, env, "read");
  }
  for (const fresh of env.moduleFresh) {
    markEscaped(state, fresh);
  }
  flushPending(state, env);
  return state.violations;
}

function visit(state, node, scope, role) {
  if (!node || typeof node.type !== "string") {
    return unknown();
  }
  if (node.type === "TSModuleDeclaration") {
    return visitNamespace(state, node, scope);
  }
  if (node.type.startsWith("TS") && !TS_EXPRESSIONS.has(node.type)) {
    return unknown();
  }
  switch (node.type) {
    case "ExpressionStatement":
      return visit(state, node.expression, scope, role);
    case "ChainExpression":
    case "ParenthesizedExpression":
    case "TSAsExpression":
    case "TSNonNullExpression":
    case "TSSatisfiesExpression":
    case "TSTypeAssertion":
      return visit(state, node.expression, scope, role);
    case "BlockStatement": {
      const block = child(scope);
      visitStatements(state, node.body, block);
      return unknown();
    }
    case "StaticBlock":
      return visitStatements(state, node.body, scope);
    case "IfStatement":
      visit(state, node.test, scope, "read");
      visit(state, node.consequent, scope, "read");
      visit(state, node.alternate, scope, "read");
      return unknown();
    case "SwitchStatement":
      visit(state, node.discriminant, scope, "read");
      for (const item of node.cases) {
        if (item.test) {
          visit(state, item.test, scope, "read");
        }
        visitStatements(state, item.consequent, scope);
      }
      return unknown();
    case "WhileStatement":
    case "DoWhileStatement":
    case "ForStatement":
    case "ForInStatement":
    case "ForOfStatement":
      return visitLoop(state, node, scope);
    case "TryStatement":
      visit(state, node.block, scope, "read");
      if (node.handler) {
        const caught = child(scope);
        if (node.handler.param) {
          bindPattern(state, node.handler.param, unknown(), caught, "local");
        }
        visit(state, node.handler.body, caught, "read");
      }
      if (node.finalizer) {
        visit(state, node.finalizer, scope, "read");
      }
      return unknown();
    case "ThrowStatement":
      visit(state, node.argument, scope, "escape");
      return unknown();
    case "ReturnStatement":
      if (node.argument) {
        visit(state, node.argument, scope, "escape");
      }
      return unknown();
    case "LabeledStatement":
      return visit(state, node.body, scope, role);
    case "ExportNamedDeclaration":
      if (node.declaration) {
        visit(state, node.declaration, scope, "read");
      }
      for (const specifier of node.specifiers) {
        escapeExport(state, specifier.local, scope);
      }
      return unknown();
    case "ExportDefaultDeclaration":
      visit(state, node.declaration, scope, "escape");
      return unknown();
    case "ExportAllDeclaration":
    case "ImportDeclaration":
    case "BreakStatement":
    case "ContinueStatement":
    case "DebuggerStatement":
    case "EmptyStatement":
      return unknown();
    case "VariableDeclaration":
      for (const declarator of node.declarations) {
        const init = declarator.init ? visit(state, declarator.init, scope, "read") : unknown();
        bindPattern(state, declarator.id, init, scope, node.kind === "var" ? "var" : "local");
      }
      return unknown();
    case "FunctionDeclaration":
    case "FunctionExpression":
    case "ArrowFunctionExpression":
      return visitFunction(state, node, scope, role);
    case "ClassDeclaration":
    case "ClassExpression":
      return visitClassNode(state, node, scope, role);
    default: {
      const expressed = visitExpression(state, node, scope, role);
      if (expressed !== undefined) {
        return expressed;
      }
      walkChildren(state, node, scope);
      return unknown();
    }
  }
}

function visitStatements(state, statements, scope) {
  for (const statement of statements) {
    visit(state, statement, scope, "read");
  }
  return unknown();
}

api.visit = visit;
api.visitStatements = visitStatements;

module.exports = { scanMutations };
