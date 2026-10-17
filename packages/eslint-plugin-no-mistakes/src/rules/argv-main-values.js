"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable, memberPropertyName, isLocalRequire } = require("./async-target-bindings");
const { literalString } = require("./module-mock-helpers");

function globalName(node, name, context) {
  if (node?.type !== "Identifier" || node.name !== name) return false;
  return !resolveVariable(node, context)?.defs.length;
}
function nodeProcess(node, context) {
  if (globalName(node, "process", context)) return true;
  if (node?.type !== "Identifier") return false;
  const variable = resolveVariable(node, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type === "ImportBinding") {
    return (
      ["ImportDefaultSpecifier", "ImportNamespaceSpecifier"].includes(def.node.type) &&
      ["process", "node:process"].includes(def.parent.source.value)
    );
  }
  const init = unwrapExpression(def?.node.init);
  return (
    def?.parent?.kind === "const" &&
    def.node.id.type === "Identifier" &&
    init?.type === "CallExpression" &&
    init.callee.type === "Identifier" &&
    init.callee.name === "require" &&
    !isLocalRequire(init.callee, context) &&
    ["process", "node:process"].includes(literalString(init.arguments[0]))
  );
}
const ARGV = 1;
const MAIN = 2;
const PROCESS = 3;
const ARGUMENTS = 4;
function createClassifier(context, wrappers) {
  const stable = new WeakMap();
  const aliases = new WeakMap();
  function classify(input, depth = 0) {
    const node = unwrapExpression(input);
    if (!node || depth > 32) return 0;
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      if (!variable) {
        if (nodeProcess(node, context)) return PROCESS;
        return globalName(node, "__filename", context) ? MAIN : 0;
      }
      if (!stable.has(variable))
        stable.set(variable, !variable.references.some((r) => r.isWrite() && !r.init));
      if (!stable.get(variable)) return 0;
      if (nodeProcess(node, context)) return PROCESS;
      if (globalName(node, "__filename", context)) return MAIN;
      const def = variable.defs.length === 1 ? variable.defs[0] : null;
      if (
        def?.type !== "Variable" ||
        def.parent.kind !== "const" ||
        def.node.id.type !== "Identifier" ||
        def.node.range[1] > node.range[0]
      )
        return 0;
      if (!aliases.has(variable)) {
        aliases.set(variable, 0);
        aliases.set(variable, classify(def.node.init, depth + 1));
      }
      return aliases.get(variable);
    }
    if (node.type === "MemberExpression") {
      const name = memberPropertyName(node);
      if (
        node.object.type === "MetaProperty" &&
        node.object.meta.name === "import" &&
        node.object.property.name === "meta" &&
        ["url", "filename"].includes(name)
      )
        return MAIN;
      const object = classify(node.object, depth + 1);
      if (object === PROCESS && name === "argv") return ARGUMENTS;
      if (
        object === ARGUMENTS &&
        node.computed &&
        node.property.type === "Literal" &&
        node.property.value === 1
      )
        return ARGV;
      // A URL conversion's href preserves the same module/argv identity.
      if (
        name === "href" &&
        unwrapExpression(node.object).type === "CallExpression" &&
        wrappers.resolveCallTarget(unwrapExpression(node.object))?.calleeName === "pathToFileURL"
      )
        return object;
    }
    if (node.type === "CallExpression") {
      const callee = unwrapExpression(node.callee);
      if (
        callee?.type === "MemberExpression" &&
        memberPropertyName(callee) === "at" &&
        classify(callee.object, depth + 1) === ARGUMENTS &&
        node.arguments.length === 1 &&
        node.arguments[0].type === "Literal" &&
        node.arguments[0].value === 1
      )
        return ARGV;
      if (node.arguments.length !== 1 || node.arguments[0].type === "SpreadElement") return 0;
      if (globalName(callee, "Boolean", context) || wrappers.resolveCallTarget(node)) {
        const value = classify(node.arguments[0], depth + 1);
        return value === ARGV || value === MAIN ? value : 0;
      }
    }
    return 0;
  }
  return (node) => classify(node);
}
module.exports = { createClassifier, ARGV, MAIN };
