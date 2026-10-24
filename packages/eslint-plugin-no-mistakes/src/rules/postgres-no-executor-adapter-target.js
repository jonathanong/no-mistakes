"use strict";

const { matchesConfiguredModule } = require("./postgres-scoped-executors");
const { resolveVariable } = require("./postgres-query-text");
const { constInitializer, stableFunction } = require("./postgres-no-executor-adapter-binding");

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

function localTarget(fn, index, property, context, options) {
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
  if (target.type !== "ObjectPattern") return false;
  const bound = target.properties.some(
    (entry) =>
      entry.type === "Property" &&
      !entry.computed &&
      entry.key.name === property &&
      entry.value.type === "Identifier",
  );
  const type = target.typeAnnotation?.typeAnnotation;
  const member =
    type?.type === "TSTypeLiteral" &&
    type.members.find(
      (entry) =>
        entry.type === "TSPropertySignature" && !entry.computed && entry.key.name === property,
    );
  return Boolean(bound && configuredType(member?.typeAnnotation?.typeAnnotation, context, options));
}

function importedTarget(call, index, property, context, options) {
  const services = context.sourceCode.parserServices;
  if (!services.program || !services.esTreeNodeToTSNodeMap) return false;
  const checker = services.program.getTypeChecker();
  const node = services.esTreeNodeToTSNodeMap.get(call.callee);
  const signatures = checker.getTypeAtLocation(node).getCallSignatures();
  if (signatures.length !== 1) return false;
  const declaration = signatures[0].getDeclaration();
  // Contextual function types describe a contract, not the implementation
  // parameter that SQL analysis judges. TypeScript function declarations own
  // asteriskToken even when undefined; method signatures do not.
  if (!declaration?.body && !(declaration?.name && "asteriskToken" in declaration)) return false;
  const functionSymbol = declaration?.name && checker.getSymbolAtLocation(declaration.name);
  if (functionSymbol?.declarations.length > 1) return false;
  const params = declaration?.parameters;
  if (!params || params.some((param) => param.dotDotDotToken)) return false;
  let type = params[index]?.type;
  if (property) {
    const pattern = params[index]?.name;
    const bound =
      pattern?.getText().startsWith("{") &&
      pattern.elements.some(
        (element) =>
          typeof element.name.text === "string" &&
          !element.initializer &&
          (!element.propertyName || typeof element.propertyName.text === "string") &&
          (element.propertyName?.text ?? element.name.text) === property &&
          !element.dotDotDotToken,
      );
    if (!bound) return false;
    type = type?.members?.find((member) => member.name?.text === property)?.type;
  }
  // Structural compatibility and inference do not prove an executor contract.
  if (!type?.typeName) return false;
  const symbol = checker.getSymbolAtLocation(type.typeName);
  return Boolean(
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
}

function acceptsExecutor(call, index, quoteNode, context, options) {
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
  if (fn) return localTarget(fn, index, property, context, options);
  const variable = resolveVariable(call.callee, context);
  if (!variable?.defs.some((def) => def.type === "ImportBinding")) return false;
  return importedTarget(call, index, property, context, options);
}

module.exports = { acceptsExecutor };
