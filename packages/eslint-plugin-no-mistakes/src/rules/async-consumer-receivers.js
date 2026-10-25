"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");
const { localFunction } = require("./async-consumer-local-proof");
const { importedFunction } = require("./async-consumer-imported-proof");

// Another dependency method receives the same object as `this`. Only ignore that
// invocation when its supplied function cannot inspect or escape its receiver.
function createReceiverProof(state) {
  const facts = new WeakMap();
  function safe(value, context, parameter, inherited, supplied) {
    value = unwrapExpression(value);
    if (value.type === "LogicalExpression" && value.operator === "??")
      return safe(value.left, context, parameter, inherited, supplied);
    if (
      value.type === "MemberExpression" &&
      value.object.type === "Identifier" &&
      resolveVariable(value.object, context) === parameter
    )
      return inherited.get(memberPropertyName(value)) === true;
    let fn = value;
    if (value.type === "Identifier") {
      fn = localFunction(value, context);
      if (!fn) {
        const imported = importedFunction(value, context, state);
        if (!imported) return false;
        fn = imported.fn;
        context = imported.context;
      }
    }
    if (!["ArrowFunctionExpression", "FunctionExpression", "FunctionDeclaration"].includes(fn.type))
      return false;
    if (!facts.has(fn)) {
      let valid = true;
      const functions = new Set();
      const variables = new Set();
      function visit(node) {
        if (!node || !valid) return;
        if (
          node.type === "ThisExpression" ||
          (node.type === "Identifier" && ["eval", "arguments"].includes(node.name))
        )
          valid = false;
        if (node.type === "Identifier") {
          variables.add(resolveVariable(node, context));
          const referenced = localFunction(node, context);
          if (referenced) functions.add(referenced);
        }
        for (const key of context.sourceCode.visitorKeys[node.type]) {
          const child = node[key];
          if (Array.isArray(child)) child.forEach(visit);
          else if (child) visit(child);
        }
      }
      fn.params.forEach(visit);
      visit(fn.body);
      facts.set(fn, { valid, functions, variables });
    }
    const proof = facts.get(fn);
    return (
      proof.valid &&
      (parameter === null || !proof.variables.has(parameter)) &&
      ![...supplied].some((callback) => proof.functions.has(callback))
    );
  }
  return function shape(object, context, parameter = null, inherited = new Map()) {
    const result = new Map();
    // A this-free sibling can still close over a supplied callback and invoke it
    // without adopting its promise. Resolve stable aliases to the same identity.
    const supplied = new Set();
    for (const entry of object.properties) {
      if (entry.type !== "Property") continue;
      const value = unwrapExpression(entry.value);
      const fn = value.type === "Identifier" ? localFunction(value, context) : value;
      if (fn) supplied.add(fn);
    }
    for (const entry of object.properties) {
      if (entry.type === "SpreadElement") {
        for (const [name, safe] of inherited) result.set(name, safe);
      } else {
        const name = entry.key.name ?? entry.key.value;
        result.set(name, safe(entry.value, context, parameter, inherited, supplied));
      }
    }
    return result;
  };
}
module.exports = { createReceiverProof };
