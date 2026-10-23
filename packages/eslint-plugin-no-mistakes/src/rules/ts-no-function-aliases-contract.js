"use strict";

const { staticMemberName } = require("./ts-no-function-aliases-ast");
const { resolveVariable } = require("./ts-no-function-aliases-self");

function unassertedPath(expression) {
  let current = expression;
  while (["ChainExpression", "MemberExpression", "AwaitExpression"].includes(current.type)) {
    if (current.type === "ChainExpression") current = current.expression;
    else current = current.type === "AwaitExpression" ? current.argument : current.object;
  }
  return !current.type.startsWith("TS");
}

function unassertedResult(node) {
  let result = node.body;
  if (result.type === "BlockStatement") {
    const statement = result.body[0];
    result = statement.argument || statement.expression;
  }
  return unassertedPath(result);
}

function legacyIdentifierCall(node, call) {
  let callee = call.callee;
  while (callee.type === "ChainExpression") callee = callee.expression;
  if (callee.type !== "Identifier" || !unassertedResult(node)) return false;
  // Template-key assignments became named wrappers in #2516, so they still
  // use the expanded-path proof even when their body calls an identifier.
  const left = node.parent?.type === "AssignmentExpression" ? node.parent.left : null;
  return !(left?.type === "MemberExpression" && left.computed && left.property.type !== "Literal");
}

function unassertedCall(node, call) {
  if (call.typeArguments || call.typeParameters) return false;
  if (!unassertedResult(node) || !unassertedPath(call.callee)) return false;
  const callee = call.callee.type === "ChainExpression" ? call.callee.expression : call.callee;
  const method = callee.type === "MemberExpression" ? staticMemberName(callee) : null;
  return (
    !["call", "apply"].includes(method) || !call.arguments[0] || unassertedPath(call.arguments[0])
  );
}

function wrapperHasOverloads(node, sourceCode) {
  if (node.type !== "FunctionDeclaration") return false;
  return sourceCode
    .getDeclaredVariables(node)
    .some((variable) =>
      variable.defs.some(
        (definition) => definition.type === "FunctionName" && definition.node !== node,
      ),
    );
}

function directIntrinsic(target, sourceCode) {
  if (target.type !== "MemberExpression") return false;
  const name = staticMemberName(target);
  if (name !== "call" && name !== "apply") return false;
  // A namespace is not callable: namespace.call is an actual module export.
  // Other imported values could expose Function.prototype.call/apply instead.
  const root = target.object;
  if (root.type !== "Identifier") return true;
  const variable = resolveVariable(sourceCode, root);
  return variable?.defs[0]?.node.type !== "ImportNamespaceSpecifier";
}

function documentedSignature(node, sourceCode) {
  let current = node;
  while (true) {
    if (
      sourceCode
        .getCommentsBefore(current)
        .some(
          (comment) =>
            comment.type === "Block" &&
            comment.value.startsWith("*") &&
            /@(param|arg|argument|returns?|type|template|overload|this|satisfies)\b/.test(
              comment.value,
            ),
        )
    )
      return true;
    const parent = current.parent;
    if (
      ![
        "VariableDeclarator",
        "VariableDeclaration",
        "ExportNamedDeclaration",
        "ExportDefaultDeclaration",
      ].includes(parent?.type)
    )
      return false;
    current = parent;
  }
}

module.exports = {
  legacyIdentifierCall,
  unassertedCall,
  wrapperHasOverloads,
  directIntrinsic,
  documentedSignature,
};
