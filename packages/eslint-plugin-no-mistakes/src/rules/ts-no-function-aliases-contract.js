"use strict";

const { staticMemberName } = require("./ts-no-function-aliases-ast");
const { resolveVariable } = require("./ts-no-function-aliases-self");

function unassertedPath(expression) {
  let current = expression;
  while (current.type === "ChainExpression" || current.type === "MemberExpression") {
    current = current.type === "ChainExpression" ? current.expression : current.object;
  }
  return !current.type.startsWith("TS");
}

function unassertedCall(node, call) {
  let result = node.body;
  if (result.type === "BlockStatement") {
    const statement = result.body[0];
    result = statement.argument || statement.expression;
  }
  return unassertedPath(result) && unassertedPath(call.callee);
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

module.exports = { unassertedCall, wrapperHasOverloads, directIntrinsic, documentedSignature };
