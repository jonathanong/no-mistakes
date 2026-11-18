"use strict";

const api = require("./no-object-mutation-runtime");
const { globalBuiltin } = require("./no-object-mutation-array-proof");
const { iteratedContents } = require("./no-object-mutation-read");
const { methodName, unwrap } = require("./no-object-mutation-syntax");
const { createFresh, remember, unknown } = require("./no-object-mutation-values");

function arrayFactory(state, node, scope) {
  const callee = unwrap(node.callee);
  if (callee?.type !== "MemberExpression" || node.optional || callee.optional) return;
  const name = methodName(callee);
  const object =
    ["entries", "keys", "values"].includes(name) &&
    globalBuiltin(state, callee.object, scope, "Object");
  const from = name === "from" && globalBuiltin(state, callee.object, scope, "Array");
  if ((!object && !from) || !node.arguments.length || node.arguments[0].type === "SpreadElement")
    return;
  const argumentRole = name === "values" ? "read" : "escape";
  const source = api.visit(state, node.arguments[0], scope, argumentRole);
  for (const argument of node.arguments.slice(1)) {
    api.visit(
      state,
      argument.type === "SpreadElement" ? argument.argument : argument,
      scope,
      argumentRole,
    );
  }
  const result = name === "values" ? createFresh(scope) : unknown();
  if (name === "values") remember(result, iteratedContents(source));
  result.isArray = true;
  result.arrayProducer = true;
  return result;
}

module.exports = { arrayFactory };
