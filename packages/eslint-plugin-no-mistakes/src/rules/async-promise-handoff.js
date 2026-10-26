"use strict";

const { dirname } = require("node:path");
const { unwrapTransparentParent } = require("./async-ast");
const { compileTargets, targetMatches } = require("./async-patterns");
const { resolveVariable } = require("./async-target-bindings");

// Oxlint --type-aware does not expose parserServices.program. The program below
// exists only for that flag; plain Oxlint and untyped ESLint never build one.
const programs = new Map();
const indexes = new WeakMap();

function typeSession(context) {
  const services = context.sourceCode?.parserServices;
  if (services?.program && services.esTreeNodeToTSNodeMap) {
    if (services.program.getCompilerOptions?.().noResolve) return null;
    const checker = services.program.getTypeChecker();
    return {
      checker,
      toTs(node) {
        return services.esTreeNodeToTSNodeMap.get(unwrapTransparentParent(node));
      },
    };
  }
  if (!process.argv.includes("--type-aware")) return null;
  return oxlintSession(context);
}

function loadProgram(configPath) {
  if (programs.has(configPath)) return programs.get(configPath);
  const ts = require("typescript");
  const read = ts.readConfigFile(configPath, ts.sys.readFile);
  const parsed =
    !read.error &&
    ts.parseJsonConfigFileContent(read.config, ts.sys, dirname(configPath), undefined, configPath);
  const fatal = parsed && parsed.errors.some((error) => error.code !== 18003);
  const program =
    parsed && !fatal && !parsed.options.noResolve
      ? ts.createProgram({ rootNames: parsed.fileNames, options: parsed.options })
      : null;
  programs.set(configPath, program);
  return program;
}

function nodesStartingAt(sourceFile, start) {
  let index = indexes.get(sourceFile);
  if (!index) {
    index = new Map();
    const visit = (node) => {
      const key = node.getStart(sourceFile);
      const list = index.get(key);
      if (list) list.push(node);
      else index.set(key, [node]);
      node.forEachChild(visit);
    };
    visit(sourceFile);
    indexes.set(sourceFile, index);
  }
  return index.get(start) || null;
}

function oxlintSession(context) {
  const filename = context.filename;
  const ts = require("typescript");
  const configPath = ts.findConfigFile(dirname(filename), ts.sys.fileExists);
  const program = configPath && loadProgram(configPath);
  const sourceFile = program && program.getSourceFile(filename);
  if (!sourceFile) return null;
  return {
    checker: program.getTypeChecker(),
    toTs(node) {
      const value = unwrapTransparentParent(node);
      const range = value?.range;
      const list = range && nodesStartingAt(sourceFile, range[0]);
      return list?.find((candidate) => candidate.end === range[1]) || null;
    },
  };
}

function isAnyLike(type, ts) {
  return Boolean(type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown));
}

function returnsThenable(type, checker, ts) {
  if (isAnyLike(type, ts)) return false;
  const signatures = type.getCallSignatures();
  if (signatures.length === 0) return false;
  return signatures.every((signature) =>
    Boolean(checker.getPromisedTypeOfPromise(checker.getReturnTypeOfSignature(signature))),
  );
}

function importsTarget(type, sourceFile, session, context) {
  const symbol = type.symbol;
  if (!symbol || !sourceFile || symbol.name === "__type") return false;
  const ts = require("typescript");
  const checker = session.checker;
  const targets = compileTargets(context.options?.[0] || {}, "targets");
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier))
      continue;
    const named = statement.importClause?.namedBindings;
    if (!named || !ts.isNamedImports(named)) continue;
    for (const specifier of named.elements) {
      const imported = (specifier.propertyName || specifier.name).text;
      if (!targetMatches(targets, statement.moduleSpecifier.text, imported)) continue;
      if (checker.getAliasedSymbol(checker.getSymbolAtLocation(specifier.name)) === symbol)
        return true;
    }
  }
  return false;
}

function isTargetThenable(type, sourceFile, session, context) {
  const ts = require("typescript");
  return (
    returnsThenable(type, session.checker, ts) && importsTarget(type, sourceFile, session, context)
  );
}

function explicitDeclaredType(node, session, context) {
  const variable = resolveVariable(unwrapTransparentParent(node), context);
  // A transparent wrapper (an assertion around the callback) has no binding.
  if (!variable) return null;
  for (const definition of variable.defs) {
    const annotation = definition.name?.typeAnnotation?.typeAnnotation;
    const tsNode = annotation && session.toTs(annotation);
    const type = tsNode && session.checker.getTypeFromTypeNode(tsNode);
    if (type) return type;
  }
  return null;
}

function sameTargetType(slot, declared, checker) {
  return checker.isTypeAssignableTo(slot, declared) && checker.isTypeAssignableTo(declared, slot);
}

// A returned target call is disposed only when the receiving slot is a configured
// target function. Callers of that type are checked again at each invocation.
function promiseSlotHandoff(node, context) {
  const session = typeSession(context);
  if (!session) return false;
  const ts = require("typescript");
  const slotNode = session.toTs(node);
  const slot = slotNode && session.checker.getContextualType(slotNode);
  if (!slot || isAnyLike(slot, ts)) return false;
  const sourceFile = slotNode.getSourceFile();
  const declared = explicitDeclaredType(node, session, context);
  if (
    declared &&
    !isAnyLike(declared, ts) &&
    isTargetThenable(declared, sourceFile, session, context)
  ) {
    return sameTargetType(slot, declared, session.checker);
  }
  return isTargetThenable(slot, sourceFile, session, context);
}

function promiseTypedTargetCall(node, context) {
  const session = typeSession(context);
  if (!session) return false;
  const callee = session.toTs(node.callee);
  if (!callee) return false;
  return isTargetThenable(
    session.checker.getTypeAtLocation(callee),
    callee.getSourceFile(),
    session,
    context,
  );
}

module.exports = { promiseSlotHandoff, promiseTypedTargetCall };
