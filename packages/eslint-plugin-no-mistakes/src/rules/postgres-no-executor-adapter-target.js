"use strict";

const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const { matchesConfiguredModule } = require("./postgres-scoped-executors");
const { resolveVariable } = require("./postgres-query-text");
const { constInitializer, stableFunction } = require("./postgres-no-executor-adapter-binding");
const { declaredOptionType } = require("./postgres-no-executor-adapter-options");
const { localOptionBody } = require("./postgres-no-executor-adapter-option-local");
const { tsOptionBody } = require("./postgres-no-executor-adapter-option-ts");
const { allowsCollectionKey } = require("./postgres-no-executor-adapter-collection");

function localFunction(callee, context) {
  const variable = resolveVariable(callee, context);
  if (
    !variable ||
    variable.references.some((ref) => ref.isWrite() && !ref.init) ||
    variable.defs.some((def) => def.node.type === "TSDeclareFunction")
  )
    return null;
  const init = constInitializer(callee, context, new Set());
  if (init?.type === "ArrowFunctionExpression" || init?.type === "FunctionExpression") return init;
  return stableFunction(callee, context);
}

function configuredType(type, context, options) {
  if (type?.type !== "TSTypeReference" || type.typeName.type !== "Identifier") return false;
  const variable = resolveVariable(type.typeName, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  return Boolean(
    def?.type === "ImportBinding" &&
    def.node.type === "ImportSpecifier" &&
    options.executorTypeNames.includes(def.node.imported.name) &&
    matchesConfiguredModule(def.parent.source.value, options.importSpecifier, context.filename),
  );
}

function localTarget(fn, index, property, context, options, facts) {
  if (fn.params.some((param) => param.type === "RestElement")) return false;
  const param = fn.params[index];
  if (!param) return false;
  const target = param.type === "AssignmentPattern" ? param.left : param;
  if (!property) {
    return (
      target.type === "Identifier" &&
      configuredType(target.typeAnnotation?.typeAnnotation, context, options)
    );
  }
  if (target.type === "Identifier") {
    const services = context.sourceCode.parserServices;
    const annotation = target.typeAnnotation?.typeAnnotation;
    if (!services.program || services.program.getCompilerOptions().noResolve || !annotation)
      return false;
    const node = services.esTreeNodeToTSNodeMap?.get(annotation);
    if (!declaredOptionType(node, property, services.program.getTypeChecker(), options))
      return false;
    const implementation = services.esTreeNodeToTSNodeMap.get(fn);
    return tsOptionBody(implementation, index, property, services.program.getTypeChecker(), facts);
  }
  if (target.type !== "ObjectPattern") return false;
  const bound = target.properties.find(
    (entry) =>
      entry.type === "Property" &&
      !entry.computed &&
      entry.key.name === property &&
      entry.value.type === "Identifier",
  );
  if (!bound) return false;
  const services = context.sourceCode.parserServices;
  if (services.program && !services.program.getCompilerOptions().noResolve) {
    const annotation = target.typeAnnotation?.typeAnnotation;
    const node = annotation && services.esTreeNodeToTSNodeMap?.get(annotation);
    if (!declaredOptionType(node, property, services.program.getTypeChecker(), options))
      return false;
    const implementation = services.esTreeNodeToTSNodeMap.get(fn);
    return tsOptionBody(implementation, index, property, services.program.getTypeChecker(), facts);
  }
  const type = target.typeAnnotation?.typeAnnotation;
  const member =
    type?.type === "TSTypeLiteral" &&
    type.members.find(
      (entry) =>
        entry.type === "TSPropertySignature" &&
        !entry.computed &&
        !entry.optional &&
        entry.key.name === property,
    );
  return (
    configuredType(member?.typeAnnotation?.typeAnnotation, context, options) &&
    localOptionBody(fn, bound.value, context, facts)
  );
}

function importedTarget(call, index, property, context, options, facts) {
  const services = context.sourceCode.parserServices;
  if (
    !services.program ||
    services.program.getCompilerOptions().noResolve ||
    !services.esTreeNodeToTSNodeMap
  )
    return false;
  const checker = services.program.getTypeChecker();
  const node = services.esTreeNodeToTSNodeMap.get(call.callee);
  const signatures = checker.getTypeAtLocation(node).getCallSignatures();
  if (signatures.length !== 1) return false;
  const declaration = signatures[0].getDeclaration();
  if (!declaration?.body) return false;
  // The signature must belong to this callee's declaration or const initializer.
  // A contextual method type can point at a different, fully annotated body.
  const target = checker.getAliasedSymbol(checker.getSymbolAtLocation(node));
  if (
    !target.declarations?.some(
      (candidate) =>
        candidate === declaration ||
        (candidate.initializer === declaration &&
          candidate.parent.getFirstToken().getText() === "const"),
    )
  )
    return false;
  const functionSymbol = declaration?.name && checker.getSymbolAtLocation(declaration.name);
  if (functionSymbol?.declarations.length > 1) return false;
  const params = declaration?.parameters;
  if (!params || params.some((param) => param.dotDotDotToken)) return false;
  const type = params[index]?.type;
  if (property) {
    const pattern = params[index]?.name;
    const bound =
      typeof pattern?.escapedText === "string" ||
      (pattern?.getText().startsWith("{") &&
        pattern.elements.some(
          (element) =>
            typeof element.name.text === "string" &&
            !element.initializer &&
            (!element.propertyName || typeof element.propertyName.text === "string") &&
            (element.propertyName?.text ?? element.name.text) === property &&
            !element.dotDotDotToken,
        ));
    if (!bound || !declaredOptionType(type, property, checker, options)) return false;
    const writes = sourceWrites(declaration.getSourceFile(), checker, facts, services.program);
    return (
      !writes.eval &&
      !writes.symbols.has(target) &&
      tsOptionBody(declaration, index, property, checker, facts)
    );
  }
  // Structural compatibility and inference do not prove an executor contract.
  if (!type?.typeName) return false;
  const symbol = checker.getSymbolAtLocation(type.typeName);
  const configured = Boolean(
    symbol?.declarations?.some((declaration) => {
      const source = declaration.parent?.parent?.parent?.moduleSpecifier?.text;
      const name = declaration.propertyName?.text ?? declaration.name?.text;
      return (
        typeof source === "string" &&
        options.executorTypeNames.includes(name) &&
        matchesConfiguredModule(source, options.importSpecifier, type.getSourceFile().fileName)
      );
    }),
  );
  if (!configured) return false;
  const writes = sourceWrites(declaration.getSourceFile(), checker, facts, services.program);
  return !writes.eval && !writes.symbols.has(target);
}

function acceptsExecutor(call, index, quoteNode, context, options, facts) {
  if (!quoteNode && allowsCollectionKey(call, index, context, facts)) return true;
  if (
    call.type !== "CallExpression" ||
    call.callee.type !== "Identifier" ||
    call.arguments.some((argument) => argument.type === "SpreadElement")
  )
    return false;
  if (
    quoteNode &&
    (quoteNode.type !== "Property" || quoteNode.computed || quoteNode.key.type !== "Identifier")
  )
    return false;
  const property = quoteNode?.key.name;
  const fn = localFunction(call.callee, context);
  if (fn) return localTarget(fn, index, property, context, options, facts);
  const variable = resolveVariable(call.callee, context);
  if (!variable?.defs.some((def) => def.type === "ImportBinding")) return false;
  return importedTarget(call, index, property, context, options, facts);
}

module.exports = { acceptsExecutor };
