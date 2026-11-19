"use strict";

const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const { resolveVariable } = require("./postgres-query-text");
const { constInitializer, stableFunction } = require("./postgres-no-executor-adapter-binding");
const { declaredOptionType } = require("./postgres-no-executor-adapter-options");
const { localOptionBody } = require("./postgres-no-executor-adapter-option-local");
const { tsOptionBody } = require("./postgres-no-executor-adapter-option-ts");
const { allowsCollectionKey } = require("./postgres-no-executor-adapter-collection");
const { localExecutorType, importedExecutorType } = require("./postgres-no-executor-adapter-type");
const {
  localMemberFunction,
  staticHelperName,
} = require("./postgres-no-executor-adapter-local-member");
const { namespaceImport } = require("./postgres-no-executor-adapter-member");

function localFunction(callee, context, facts) {
  if (callee.type === "MemberExpression") return localMemberFunction(callee, context, facts);
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

function localTarget(fn, index, property, context, options, facts) {
  if (fn.params.some((param) => param.type === "RestElement")) return false;
  const param = fn.params[index];
  if (!param) return false;
  const target = param.type === "AssignmentPattern" ? param.left : param;
  if (!property) {
    return (
      target.type === "Identifier" &&
      localExecutorType(target.typeAnnotation?.typeAnnotation, context, options)
    );
  }
  if (target.type === "Identifier") {
    const services = context.sourceCode.parserServices;
    const annotation = target.typeAnnotation?.typeAnnotation;
    if (!services.program || services.program.getCompilerOptions().noResolve || !annotation)
      return false;
    const node = services.esTreeNodeToTSNodeMap?.get(annotation);
    if (
      !declaredOptionType(
        node,
        property,
        services.program.getTypeChecker(),
        options,
        param.type === "AssignmentPattern",
      )
    )
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
    localExecutorType(member?.typeAnnotation?.typeAnnotation, context, options) &&
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
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const node = services.esTreeNodeToTSNodeMap.get(call.callee);
  if (!node) return false;
  const signatures = checker.getTypeAtLocation(node).getCallSignatures();
  if (signatures.length !== 1) return false;
  const declaration = signatures[0].getDeclaration();
  if (!declaration?.body) return false;
  // The signature must belong to this export's declaration or const initializer.
  // A contextual method type can point at a different, fully annotated body.
  const symbol = checker.getSymbolAtLocation(node.argumentExpression ?? node);
  if (!symbol) return false;
  const target = symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
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
    if (
      !bound ||
      !declaredOptionType(type, property, checker, options, Boolean(params[index].initializer))
    )
      return false;
    const writes = sourceWrites(declaration.getSourceFile(), checker, facts, services.program);
    return (
      !writes.eval &&
      !writes.symbols.has(target) &&
      tsOptionBody(declaration, index, property, checker, facts)
    );
  }
  // Structural compatibility and inference do not prove an executor contract.
  if (!type || !importedExecutorType(type, checker, options)) return false;
  const writes = sourceWrites(declaration.getSourceFile(), checker, facts, services.program);
  return !writes.eval && !writes.symbols.has(target);
}

function isImportedHelper(callee, context) {
  if (callee.type === "MemberExpression") {
    return staticHelperName(callee) !== null && namespaceImport(callee.object, context);
  }
  return (
    resolveVariable(callee, context)?.defs.some((def) => def.type === "ImportBinding") ?? false
  );
}

function acceptsExecutor(call, index, quoteNode, context, options, facts) {
  if (!quoteNode && allowsCollectionKey(call, index, context, facts)) return true;
  if (
    call.type !== "CallExpression" ||
    call.arguments.some((argument) => argument.type === "SpreadElement")
  )
    return false;
  if (
    quoteNode &&
    (quoteNode.type !== "Property" || quoteNode.computed || quoteNode.key.type !== "Identifier")
  )
    return false;
  const property = quoteNode?.key.name;
  const callee = call.callee;
  const fn = localFunction(callee, context, facts);
  if (fn) return localTarget(fn, index, property, context, options, facts);
  if (!isImportedHelper(callee, context)) return false;
  return importedTarget(call, index, property, context, options, facts);
}

module.exports = { acceptsExecutor, isImportedHelper, localFunction };
