"use strict";

const { staticMemberName, unwrapExpression } = require("./ts-no-function-aliases-ast");
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

function moduleObject(root, sourceCode) {
  if (root?.type !== "Identifier") return false;
  const def = resolveVariable(sourceCode, root)?.defs[0];
  if (!def) return false;
  if (def.node?.type === "ImportNamespaceSpecifier") return true;
  if (def.node?.type === "TSImportEqualsDeclaration" && def.node.importKind !== "type") return true;
  if (
    def.type !== "Variable" ||
    def.parent?.kind !== "const" ||
    def.node?.id?.type !== "Identifier"
  ) {
    return false;
  }
  const call = unwrapExpression(def.node.init);
  const callee = unwrapExpression(call?.callee);
  return (
    call?.type === "CallExpression" &&
    callee?.type === "Identifier" &&
    callee.name === "require" &&
    !resolveVariable(sourceCode, callee)?.defs.length
  );
}

function directIntrinsic(target, sourceCode) {
  if (target.type !== "MemberExpression") return false;
  const name = staticMemberName(target);
  if (name !== "call" && name !== "apply") return false;
  // A namespace or require() module is not callable: .call is a module export.
  const root = unwrapExpression(target.object);
  if (root?.type !== "Identifier") return true;
  return !moduleObject(root, sourceCode);
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
