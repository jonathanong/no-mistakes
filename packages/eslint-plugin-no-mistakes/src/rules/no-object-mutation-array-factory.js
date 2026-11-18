"use strict";

const api = require("./no-object-mutation-runtime");
const { globalBuiltin } = require("./no-object-mutation-array-proof");
const { iteratedContents } = require("./no-object-mutation-read");
const { escapeResult } = require("./no-object-mutation-result");
const { methodName, unwrap } = require("./no-object-mutation-syntax");
const { createFresh, remember, shared, unknown } = require("./no-object-mutation-values");

function arrayOf(scope, contents) {
  const array = createFresh(scope);
  array.isArray = true;
  remember(array, contents);
  return array;
}

function arrayFactory(state, node, scope, role) {
  const callee = unwrap(node.callee);
  if (callee?.type !== "MemberExpression" || node.optional || callee.optional) return;
  const name = methodName(callee);
  const object =
    ["entries", "keys", "values"].includes(name) &&
    globalBuiltin(state, callee.object, scope, "Object");
  const from = name === "from" && globalBuiltin(state, callee.object, scope, "Array");
  if ((!object && !from) || !node.arguments.length || node.arguments[0].type === "SpreadElement")
    return;
  const source = api.visit(state, node.arguments[0], scope, "read");
  for (const argument of node.arguments.slice(1)) {
    api.visit(
      state,
      argument.type === "SpreadElement" ? argument.argument : argument,
      scope,
      from ? "escape" : "read",
    );
  }
  let contents = name === "keys" ? unknown() : iteratedContents(source);
  if (from && node.arguments.length > 1) contents = shared("parameter");
  if (name === "entries") {
    const pair = arrayOf(scope, contents);
    pair.fields.set("0", unknown());
    pair.fields.set("1", contents);
    contents = pair;
  }
  const result = arrayOf(scope, contents);
  escapeResult(state, result, scope, role);
  return result;
}

module.exports = { arrayFactory };
