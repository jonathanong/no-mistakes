"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { traverse, unwrapSequence } = require("./async-ast");
const { memberPropertyName } = require("./logical-call");

const DEFAULT_METHODS = ["isVisible", "isHidden", "isEnabled", "isChecked"];
const DEFAULT_MESSAGE =
  "A failed Playwright state check is turned into a boolean. Seed the state, then assert with expect(locator).toBeVisible() or expect(locator).toBeHidden().";

// Named handlers are not resolved. A same-file function that returns false is
// unproven, which keeps `.catch(handleError)` that rethrows silent. try/catch
// counts only a returned await, so a rejection that leaves the try is not a swallow.
// A nested try with its own catch owns that body. The outer catch is not proof
// the inner rejection became the outer boolean, even when the inner catch rethrows.
const methodList = {
  type: "array",
  items: { type: "string", minLength: 1 },
  uniqueItems: true,
};

function isStateCall(node, methods) {
  const current = unwrapSequence(node);
  if (current.type !== "CallExpression") return false;
  return methods.has(memberPropertyName(unwrapSequence(current.callee)));
}

function isBooleanLiteral(node) {
  const current = unwrapSequence(node);
  return current?.type === "Literal" && typeof current.value === "boolean";
}

function blockReturnsBoolean(context, block) {
  let found = false;
  traverse(context, block, (node) => {
    if (node.type === "ReturnStatement" && isBooleanLiteral(node.argument)) found = true;
  });
  return found;
}

function handlerReturnsBoolean(context, handler) {
  const current = unwrapSequence(handler);
  if (current.type === "ArrowFunctionExpression" && current.body.type !== "BlockStatement") {
    return isBooleanLiteral(current.body);
  }
  if (current.type !== "ArrowFunctionExpression" && current.type !== "FunctionExpression") {
    return false;
  }
  return blockReturnsBoolean(context, current.body);
}

function inTryBlock(node, tryStatement) {
  for (let cursor = node; cursor !== tryStatement; cursor = cursor.parent) {
    if (cursor === tryStatement.block) return true;
  }
  return false;
}

// A handler between this return and the outer try block catches the await.
// The handler body and a handler-less finally still propagate outward.
function shieldedByNestedTry(node, outerBlock) {
  for (let current = node.parent; current !== outerBlock; current = current.parent) {
    if (current.type === "TryStatement" && current.handler && inTryBlock(node, current)) {
      return true;
    }
  }
  return false;
}

function tryReturnsAwaitedState(context, block, methods) {
  let found = false;
  traverse(context, block, (node) => {
    if (node.type !== "ReturnStatement" || shieldedByNestedTry(node, block)) return;
    const awaited = unwrapSequence(node.argument);
    if (awaited?.type !== "AwaitExpression") return;
    if (isStateCall(awaited.argument, methods)) found = true;
  });
  return found;
}

function catchOfState(node, methods) {
  const callee = unwrapSequence(node.callee);
  if (memberPropertyName(callee) !== "catch") return null;
  if (!isStateCall(callee.object, methods)) return null;
  return node.arguments[0] ?? null;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow Playwright state checks whose failure becomes a boolean",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          methods: methodList,
          message: { type: "string", minLength: 1, maxLength: 512 },
        },
      },
    ],
    messages: { swallowed: "{{message}}" },
  },
  (context) => {
    const options = context.options[0] ?? {};
    const methods = new Set(options.methods ?? DEFAULT_METHODS);
    const message = options.message ?? DEFAULT_MESSAGE;
    const suppressed = ruleSuppression(context, "playwright-no-swallowed-visibility");
    const report = (node) => {
      if (suppressed(node)) return;
      context.report({ node, messageId: "swallowed", data: { message } });
    };
    return {
      CallExpression(node) {
        const handler = catchOfState(node, methods);
        if (!handler || !handlerReturnsBoolean(context, handler)) return;
        report(node);
      },
      TryStatement(node) {
        const handler = node.handler;
        if (!handler) return;
        if (!blockReturnsBoolean(context, handler.body)) return;
        if (!tryReturnsAwaitedState(context, node.block, methods)) return;
        report(handler);
      },
    };
  },
);
