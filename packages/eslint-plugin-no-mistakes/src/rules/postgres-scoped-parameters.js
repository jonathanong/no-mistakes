"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { variableIsImportedExecutor } = require("./postgres-no-executor-adapter-binding");

function typeMatches(annotation, types, context, options) {
  const node = annotation?.typeAnnotation ?? annotation;
  if (node?.type === "TSTypeReference") {
    return node.typeName?.type === "Identifier" && types.has(node.typeName.name);
  }
  if (node?.type === "TSTypeQuery" && node.exprName?.type === "Identifier") {
    return (
      !options.legacyAdapterScope &&
      variableIsImportedExecutor(resolveVariable(node.exprName, context), options)
    );
  }
  if (node?.type === "TSUnionType") {
    return node.types.some((member) => typeMatches(member, types, context, options));
  }
  return false;
}

function propertyKeyName(key) {
  if (key?.type === "Identifier") return key.name;
  return key?.type === "Literal" ? String(key.value) : null;
}

function declaredProperties(target, context, options) {
  if (options.executorTypeNames.length === 0) return null;
  const services = context?.sourceCode?.parserServices;
  const annotation = target.typeAnnotation?.typeAnnotation;
  // Parser autofix fallback programs cannot resolve imported option contracts.
  if (
    !services?.program ||
    services.program.getCompilerOptions().noResolve ||
    !services.esTreeNodeToTSNodeMap ||
    !annotation
  )
    return null;
  const node = services.esTreeNodeToTSNodeMap.get(annotation);
  if (!node) return null;
  const checker = services.program.getTypeChecker();
  const {
    declaredOptionType,
    optionObjectType,
  } = require("./postgres-no-executor-adapter-options");
  return {
    names: optionObjectType(node, checker)
      .getProperties()
      .map((symbol) => symbol.name),
    accepts: (property) => declaredOptionType(node, property, checker, options),
  };
}

// Syntax-only inline contracts retain legacy behavior. Named/interface/alias
// contracts require the same explicit imported type identity as the adapter.
function parameterExecutors(param, types, context, options) {
  const target = param.type === "AssignmentPattern" ? param.left : param;
  const names = [];
  const members = [];
  if (target.type === "Identifier") {
    if (typeMatches(target.typeAnnotation, types, context, options)) names.push(target.name);
    const declared = declaredProperties(target, context, options);
    for (const property of declared?.names ?? []) {
      if (declared.accepts(property)) members.push({ name: target.name, property });
    }
    return { names, members };
  }
  if (target.type !== "ObjectPattern") return { names, members };
  const literal = target.typeAnnotation?.typeAnnotation;
  const declared = declaredProperties(target, context, options);
  for (const property of target.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier") continue;
    const key = propertyKeyName(property.key);
    const inline =
      literal?.type === "TSTypeLiteral" &&
      literal.members.some(
        (member) =>
          member.type === "TSPropertySignature" &&
          propertyKeyName(member.key) === key &&
          typeMatches(member.typeAnnotation, types, context, options),
      );
    if (
      inline ||
      (!property.computed && property.value?.type === "Identifier" && declared?.accepts(key))
    ) {
      names.push(value.name);
    }
  }
  return { names, members };
}

module.exports = { parameterExecutors };
