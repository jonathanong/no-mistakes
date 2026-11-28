"use strict";

const { unwrapExpression, unwrapSequence } = require("./async-ast");
const { literalPropertyName } = require("./test-no-delayed-rejects-chains");
const { EXPECT_MODULES, isExpectCallee } = require("./test-message-expect");

function shiftArguments(method, args) {
  if (args.some((argument) => argument.type === "SpreadElement")) return null;
  if (method === "call") return args.slice(1);
  const list = unwrapExpression(args[1]);
  return list?.type === "ArrayExpression" &&
    list.elements.every((element) => element && element.type !== "SpreadElement")
    ? list.elements
    : null;
}

function forwardedMethod(callee) {
  const method = literalPropertyName(callee);
  return method === "call" || method === "apply" ? method : null;
}

function forwardedInvocation(call) {
  let callee = unwrapSequence(call.callee);
  let args = call.arguments;
  const method = forwardedMethod(callee);
  if (!method) return { callee, args };
  let receiver = args[0];
  const shifted = shiftArguments(method, args);
  callee = unwrapSequence(callee.object);
  if (!shifted) return forwardedMethod(callee) ? null : { callee, args: null };
  args = shifted;
  // Invoking Function.prototype.call/apply invokes its this value, not the
  // function from which that intrinsic was read: expect.call.call(other, ...).
  let intrinsic;
  while ((intrinsic = forwardedMethod(callee))) {
    callee = unwrapSequence(receiver);
    receiver = args[0];
    args = shiftArguments(intrinsic, args);
    if (!args) return null;
  }
  return { callee, args };
}

function expectationArguments(call, context) {
  const forwarded = forwardedInvocation(call);
  if (!forwarded) return null;
  return isExpectCallee(forwarded.callee, context, new Set(), EXPECT_MODULES, {
    allowPoll: false,
    allowForwarding: false,
  })
    ? forwarded.args
    : null;
}

function expectedIdentifier(rejects, context) {
  if (literalPropertyName(rejects) !== "rejects") return null;
  const expectation = unwrapExpression(rejects.object);
  if (expectation?.type !== "CallExpression") return null;
  const args = expectationArguments(expectation, context);
  if (args?.length !== 1) return null;
  const argument = unwrapExpression(args[0]);
  return argument.type === "Identifier" ? argument : null;
}

module.exports = { expectedIdentifier, forwardedInvocation };
