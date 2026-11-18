"use strict";

const { adjacentAnnotations } = require("../adjacent-annotation");
const { child } = require("./no-object-mutation-values");

function annotationTargets(node) {
  const targets = [node];
  let parent = node.parent;
  if (parent?.type === "Property" && parent.value === node) targets.push(parent);
  if (parent?.type === "VariableDeclarator" && parent.init === node) {
    parent = parent.parent;
    if (parent?.type === "VariableDeclaration") targets.push(parent);
  }
  if (parent?.type === "ExportNamedDeclaration" || parent?.type === "ExportDefaultDeclaration") {
    targets.push(parent);
  } else if (parent?.parent?.type === "ExportNamedDeclaration") {
    targets.push(parent.parent);
  }
  return targets;
}

function declaredReceiver(sourceCode, node) {
  if (
    node.params.some(
      (param) => param.type === "Identifier" && param.name === "this" && param.typeAnnotation,
    )
  ) {
    return true;
  }
  return annotationTargets(node).some((target) => {
    const comment = sourceCode.getCommentsBefore(target).at(-1);
    return (
      comment?.type === "Block" &&
      comment.value.startsWith("*") &&
      /(?:^|\n)\s*\*?\s*@this\s+\{\s*[^}\s][^\r\n]*\}/.test(comment.value) &&
      adjacentAnnotations(sourceCode, target, "@this").some((type) =>
        /^\{\s*[^}\s][^\r\n]*\}/.test(type),
      )
    );
  });
}

function globalArrayType(sourceCode, type) {
  if (type?.type !== "TSTypeReference" || type.typeName.name !== "Array") return false;
  const identifier = type.typeName;
  const reference = sourceCode
    .getScope(identifier)
    .references.find((item) => item.identifier === identifier);
  return !reference.resolved?.defs.length;
}

function functionEnvironment(state, node, scope, tag) {
  const env = child(scope);
  env.ownerEnv = env;
  env.functionEnv = env;
  env.fnTag = tag;
  env.pending = [];
  env.returned = [];
  const arrow = node.type === "ArrowFunctionExpression";
  const classMethod =
    node.parent?.type === "MethodDefinition" || node.parent?.type === "TSAbstractMethodDefinition";
  const declared = !arrow && declaredReceiver(state.context.sourceCode, node);
  const receiverType = node.params.find((param) => param.name === "this")?.typeAnnotation
    ?.typeAnnotation;
  const arrayReceiver =
    receiverType?.type === "TSArrayType" ||
    receiverType?.type === "TSTupleType" ||
    globalArrayType(state.context.sourceCode, receiverType) ||
    (declared &&
      annotationTargets(node).some((target) =>
        adjacentAnnotations(state.context.sourceCode, target, "@this").some((type) =>
          /^\{\s*(?:[^{}]+\[\]|Array\s*<[^{}]+>)\s*\}/.test(type),
        ),
      ));
  env.inClass = arrow ? Boolean(scope.inClass) : classMethod || declared;
  env.hasArguments = arrow ? Boolean(scope.hasArguments) : true;
  env.receiver = arrow
    ? scope.receiver || null
    : declared
      ? { type: "this", inClass: true, isArray: arrayReceiver }
      : classMethod
        ? { type: "this", inClass: true, isArray: Boolean(scope.arrayThis && !node.parent.static) }
        : tag.receiver || null;
  return env;
}

function arrayParameter(sourceCode, node) {
  const type = node.typeAnnotation?.typeAnnotation;
  return (
    type?.type === "TSArrayType" ||
    type?.type === "TSTupleType" ||
    globalArrayType(sourceCode, type)
  );
}

module.exports = { arrayParameter, functionEnvironment };
