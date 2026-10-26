"use strict";

const { staticMemberName } = require("./ts-no-function-aliases-ast");
const { resolveVariable } = require("./ts-no-function-aliases-self");
const {
  unassertedPath,
  assertedContractPreserved,
  wrapperHasOverloads,
} = require("./ts-no-function-aliases-cast-proof");

function unassertedCall(node, call) {
  if (call.typeArguments || call.typeParameters) return false;
  // Assertions are checked against the reachable target after candidate
  // selection. This guard still rejects casts on an explicit receiver.
  const callee = call.callee.type === "ChainExpression" ? call.callee.expression : call.callee;
  const method = callee.type === "MemberExpression" ? staticMemberName(callee) : null;
  return (
    !["call", "apply"].includes(method) || !call.arguments[0] || unassertedPath(call.arguments[0])
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
  unassertedCall,
  assertedContractPreserved,
  wrapperHasOverloads,
  directIntrinsic,
  documentedSignature,
};
