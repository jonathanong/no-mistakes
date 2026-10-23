"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  unwrapExpression,
  isSameArgumentList,
  callableTarget,
} = require("./ts-no-function-aliases-ast");
const { isNamedWrapper, isSelfCall, patternHasName } = require("./ts-no-function-aliases-self");
const { legacyIdentifierCall } = require("./ts-no-function-aliases-legacy");
const { exportedDeclarations, reachableTarget } = require("./ts-no-function-aliases-reachable");
const {
  unassertedCall,
  wrapperHasOverloads,
  directIntrinsic,
  documentedSignature,
} = require("./ts-no-function-aliases-contract");
const { signaturePreserved } = require("./ts-no-function-aliases-signature");
const { forwardedCall } = require("./ts-no-function-aliases-forwarding");

function expressionStatementCall(statement) {
  return statement.type === "ExpressionStatement" ? unwrapExpression(statement.expression) : null;
}

function returnStatementCall(statement) {
  return statement.type === "ReturnStatement" ? unwrapExpression(statement.argument) : null;
}

function onlyCallExpression(body) {
  if (!body) return null;
  if (body.type !== "BlockStatement") return unwrapExpression(body);
  if (body.body.length !== 1) return null;
  const statement = body.body[0];
  return returnStatementCall(statement) || expressionStatementCall(statement);
}

function reportIfAlias(node, context, suppressed, declarations) {
  if (suppressed(node)) return;
  const legacy = legacyIdentifierCall(node);
  if (legacy !== null) {
    if (legacy) context.report({ node, messageId: "alias" });
    return;
  }
  const call = onlyCallExpression(node.body);
  if (!call || call.type !== "CallExpression") return;
  if (!isNamedWrapper(node)) return;
  const sourceCode = context.sourceCode;
  if (
    !unassertedCall(node, call) ||
    wrapperHasOverloads(node, sourceCode) ||
    documentedSignature(node, sourceCode)
  )
    return;
  const forwarded = forwardedCall(call, sourceCode);
  if (isSelfCall(node, call, sourceCode, forwarded)) return;
  const directTarget = callableTarget(call.callee);
  const directForward =
    directTarget && !directIntrinsic(directTarget, sourceCode)
      ? { target: directTarget, args: call.arguments }
      : null;
  for (const candidate of [directForward, forwarded]) {
    if (!candidate) continue;
    if (!isSameArgumentList(node.params, candidate.args)) continue;
    const reachable = reachableTarget(candidate.target, sourceCode, declarations);
    if (!reachable || !signaturePreserved(node, reachable)) continue;
    context.report({ node, messageId: "alias" });
    return;
  }
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow function wrappers that only alias another function",
        recommended: true,
      },
      schema: [],
      messages: {
        alias:
          "Do not create a function that only aliases another function call. Export or call the original function name directly so agents can trace behavior.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "ts-no-function-aliases");
      let declarations;
      return {
        Program(node) {
          declarations = exportedDeclarations(node, context.sourceCode);
        },
        ArrowFunctionExpression(node) {
          reportIfAlias(node, context, suppressed, declarations);
        },
        FunctionDeclaration(node) {
          reportIfAlias(node, context, suppressed, declarations);
        },
        FunctionExpression(node) {
          reportIfAlias(node, context, suppressed, declarations);
        },
      };
    },
  ),
  { __test: { onlyCallExpression, isSelfCall, patternHasName } },
);
