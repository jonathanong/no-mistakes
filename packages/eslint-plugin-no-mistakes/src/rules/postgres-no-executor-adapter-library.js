"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { literalSql } = require("./postgres-no-executor-adapter-sql");
const { importedSpecifier } = require("./postgres-no-executor-adapter-type");
const { matchesConfiguredModule } = require("./postgres-scoped-executors");

// A configured library executor call that receives the options is a terminal;
// see "Library executor terminals" in docs/eslint-rules/postgres-no-executor-adapter.md.

function scalar(node) {
  return (
    (node.type === "Literal" && !node.regex && !node.bigint) ||
    (node.type === "TemplateLiteral" && node.expressions.length === 0)
  );
}

function scalarOverrides(object, spread) {
  return (
    object.properties[0] === spread &&
    object.properties
      .slice(1)
      .every((entry) => entry.type === "Property" && !entry.computed && scalar(entry.value))
  );
}

function configuredCallee(callee, context, options) {
  const def = callee.type === "Identifier" ? resolveVariable(callee, context)?.defs[0] : null;
  const source = def?.parent?.source?.value;
  return Boolean(
    def?.type === "ImportBinding" &&
    def.node.type === "ImportSpecifier" &&
    def.parent.importKind !== "type" &&
    def.node.importKind !== "type" &&
    options.executorNames.includes(def.node.imported.name) &&
    matchesConfiguredModule(source, options.importSpecifier, context.filename) &&
    (!options.importSpecifier || source === options.importSpecifier),
  );
}

// The call whose options argument is this binding, either bare or as the first
// spread of an object followed only by literal scalar overrides.
function optionsCall(identifier, context, options) {
  let node = identifier;
  if (node.parent.type === "SpreadElement") {
    const object = node.parent.parent;
    if (object.type !== "ObjectExpression" || !scalarOverrides(object, node.parent)) return null;
    node = object;
  }
  const call = node.parent;
  return call.type === "CallExpression" &&
    !call.optional &&
    call.arguments[1] === node &&
    !call.arguments.some((argument) => argument.type === "SpreadElement") &&
    configuredCallee(call.callee, context, options)
    ? call
    : null;
}

// The statement is the parameter of an adapter closure passed to a helper. The
// helper's own SQL is proven where that closure is judged.
function adapterStatement(statement, call, context, options) {
  const def = statement.type === "Identifier" ? resolveVariable(statement, context)?.defs[0] : null;
  const fn = def?.type === "Parameter" ? def.node : null;
  return Boolean(
    fn?.parent.type === "CallExpression" &&
    fn.parent.arguments.includes(fn) &&
    libraryAdapter(fn, context, options) === call,
  );
}

// Null when the reference is not such a terminal, else whether its SQL is visible.
function libraryVisible(identifier, context, options, safeHole) {
  const call = optionsCall(identifier, context, options);
  if (!call) return null;
  return (
    adapterStatement(call.arguments[0], call, context, options) ||
    literalSql(call.arguments[0], context, options, safeHole)
  );
}

function enclosingParameter(identifier, context) {
  const variable = resolveVariable(identifier, context);
  return Boolean(
    variable?.defs.length === 1 &&
    variable.defs[0].type === "Parameter" &&
    !variable.references.some((ref) => ref.isWrite() && !ref.init),
  );
}

function closureCall(fn) {
  const [param] = fn.params;
  if (fn.async || fn.generator || fn.params.length !== 1 || param.type !== "Identifier")
    return null;
  const statements = fn.body.type === "BlockStatement" ? fn.body.body : null;
  if (statements && (statements.length !== 1 || statements[0].type !== "ReturnStatement"))
    return null;
  const call = statements ? statements[0].argument : fn.body;
  return call?.type === "CallExpression" && call.arguments.length <= 2 ? call : null;
}

// `statement => write(statement, options)`: one expression, the parameter passed
// straight through, and options that are an unwritten parameter of the enclosing
// function, optionally spread with literal scalar overrides.
function libraryAdapter(fn, context, options) {
  const call = closureCall(fn);
  const [statement, extra] = call?.arguments ?? [];
  if (!call || call.optional || !configuredCallee(call.callee, context, options)) return null;
  if (statement?.type !== "Identifier" || statement.name !== fn.params[0].name) return null;
  if (!extra) return null;
  const spread = extra.type === "ObjectExpression" ? extra.properties[0] : null;
  const binding = spread?.type === "SpreadElement" ? spread.argument : extra;
  return binding.type === "Identifier" &&
    optionsCall(binding, context, options) === call &&
    enclosingParameter(binding, context)
    ? call
    : null;
}

function tsScalar(node, ts) {
  return (
    ts.isStringLiteral(node) ||
    ts.isNumericLiteral(node) ||
    ts.isNoSubstitutionTemplateLiteral(node) ||
    [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword, ts.SyntaxKind.NullKeyword].includes(
      node.kind,
    )
  );
}

function tsPlainObject(spread, ts) {
  const object = spread.parent;
  return (
    ts.isObjectLiteralExpression(object) &&
    object.properties[0] === spread &&
    object.properties
      .slice(1)
      .every(
        (entry) =>
          ts.isPropertyAssignment(entry) &&
          !ts.isComputedPropertyName(entry.name) &&
          tsScalar(entry.initializer, ts),
      )
  );
}

function tsLibraryCall(identifier, ts, checker, options) {
  let node = identifier;
  if (ts.isSpreadAssignment(node.parent)) {
    if (!tsPlainObject(node.parent, ts)) return null;
    node = node.parent.parent;
  }
  const call = node.parent;
  const shared = { checker, sourceFile: node.getSourceFile() };
  return ts.isCallExpression(call) &&
    !call.questionDotToken &&
    ts.isIdentifier(call.expression) &&
    call.arguments[1] === node &&
    !call.arguments.some(ts.isSpreadElement) &&
    importedSpecifier(call.expression, options.executorNames, shared, options, true)
    ? call
    : null;
}

function tsClosureCall(fn, ts) {
  if (!ts.isBlock(fn.body)) return fn.body;
  const [statement] = fn.body.statements;
  return fn.body.statements.length === 1 && ts.isReturnStatement(statement)
    ? statement.expression
    : null;
}

// The statement of a library call is the parameter of an adapter closure that
// is itself a call argument; the receiving helper's SQL is proven there.
function tsAdapterStatement(call, ts, checker) {
  const [statement] = call.arguments;
  const fn = ts.isIdentifier(statement)
    ? checker.getSymbolAtLocation(statement)?.valueDeclaration?.parent
    : null;
  return Boolean(
    fn &&
    (ts.isArrowFunction(fn) || ts.isFunctionExpression(fn)) &&
    fn.parameters.length === 1 &&
    !fn.asteriskToken &&
    !fn.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.AsyncKeyword) &&
    ts.isCallExpression(fn.parent) &&
    fn.parent.arguments.includes(fn) &&
    tsClosureCall(fn, ts) === call,
  );
}

module.exports = { libraryAdapter, libraryVisible, tsAdapterStatement, tsLibraryCall };
