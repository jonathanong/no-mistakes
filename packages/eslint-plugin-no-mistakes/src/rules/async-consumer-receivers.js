"use strict";
const { unwrapExpression } = require("./async-ast");
const {
  resolveVariable,
  memberPropertyName,
  stableConstInitializer,
} = require("./async-target-bindings");
const { logicalCopyLeft } = require("./async-consumer-copy-spellings");
const { localFunction } = require("./async-consumer-local-proof");
const { importedFunction } = require("./async-consumer-imported-proof");
const { CLOSED, createDataFacts } = require("./async-handoff-data");

// Another dependency method receives the same object as `this`. Only ignore that
// invocation when its supplied function cannot inspect or escape its receiver.
function createReceiverProof(state) {
  const facts = new WeakMap();
  const dataFacts = createDataFacts();
  function safe(value, context, parameter, inherited, supplied) {
    value = unwrapExpression(value);
    const copied = logicalCopyLeft(value);
    if (copied) return safe(copied, context, parameter, inherited, supplied);
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
      const proof = { valid: true, functions: new Set(), variables: new Set() };
      const scanned = new Set();
      function inspect(current, currentContext) {
        if (scanned.has(current)) return;
        if (scanned.size >= 32) {
          proof.valid = false;
          return;
        }
        scanned.add(current);
        function visit(node) {
          if (!node || !proof.valid) return;
          if (
            ["TSTypeAnnotation", "TSTypeAliasDeclaration", "TSInterfaceDeclaration"].includes(
              node.type,
            )
          )
            return;
          if (
            node.type === "ThisExpression" ||
            (node.type === "Identifier" && ["eval", "arguments"].includes(node.name))
          )
            proof.valid = false;
          if (["CallExpression", "NewExpression", "TaggedTemplateExpression"].includes(node.type)) {
            const callee = unwrapExpression(node.callee || node.tag);
            // Unknown callees and holders can hide another invocation of a supplied callback.
            if (node.type !== "CallExpression" || !localFunction(callee, currentContext))
              proof.valid = false;
          }
          if (node.type === "MemberExpression") {
            const object = unwrapExpression(node.object);
            const init =
              object.type === "Identifier"
                ? unwrapExpression(stableConstInitializer(object, currentContext, new Set()))
                : object;
            // An opaque property read may invoke a getter that calls another callback.
            if (
              memberPropertyName(node) !== "length" ||
              !(
                init?.type === "ArrayExpression" ||
                (init?.type === "Literal" && typeof init.value === "string")
              )
            )
              proof.valid = false;
          }
          if (node.type === "Identifier") {
            const variable = resolveVariable(node, currentContext);
            proof.variables.add(variable);
            const referenced = localFunction(node, currentContext);
            if (referenced && referenced !== current) {
              proof.functions.add(referenced);
              inspect(referenced, currentContext);
            }
            if (!referenced && variable?.defs.length) {
              let owner = variable.defs[0].node;
              while (owner && owner !== current) owner = owner.parent;
              const init = unwrapExpression(
                stableConstInitializer(node, currentContext, new Set()),
              );
              const primitive =
                init?.type === "Literal" && (init.value === null || typeof init.value !== "object");
              // Captured holders can invoke getters through spreads as well as members.
              if (!owner && !primitive) proof.valid = false;
            }
          }
          for (const key of currentContext.sourceCode.visitorKeys[node.type] || []) {
            const child = node[key];
            if (Array.isArray(child)) child.forEach(visit);
            else if (child) visit(child);
          }
        }
        current.params.forEach(visit);
        visit(current.body);
      }
      inspect(fn, context);
      proof.functions.delete(fn);
      facts.set(fn, proof);
    }
    const proof = facts.get(fn);
    return (
      proof.valid &&
      (parameter === null || !proof.variables.has(parameter)) &&
      ![...supplied].some((callback) => proof.functions.has(callback))
    );
  }
  function shape(object, context, parameter = null, inherited = new Map()) {
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
        const value = unwrapExpression(entry.value);
        result.set(
          name,
          dataFacts.date(value, context)
            ? "date"
            : safe(entry.value, context, parameter, inherited, supplied),
        );
      }
    }
    if (dataFacts.closed(object, context, inherited)) result.set(CLOSED, true);
    else result.delete(CLOSED);
    return result;
  }
  shape.dataFacts = dataFacts;
  return shape;
}
module.exports = { createReceiverProof };
