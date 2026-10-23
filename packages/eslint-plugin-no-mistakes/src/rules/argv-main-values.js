"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable, memberPropertyName, isLocalRequire } = require("./async-target-bindings");
const { literalString, logicalCall } = require("./module-mock-helpers");
const { ARGV, identity, concatenate } = require("./argv-main-fragments");

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
const MAIN = 2;
const PROCESS = 3;
const ARGUMENTS = 4;
function createClassifier(context, wrappers) {
  const stable = new WeakMap();
  const aliases = new WeakMap();
  const value = (node, depth) => identity(classify(node, depth));
  function classify(input, depth = 0) {
    const node = unwrapExpression(input);
    if (!node || depth > 32) return 0;
    const text = literalString(node);
    if (text !== null) return text;
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
      if (def?.type === "ImportBinding" && def.node?.type === "ImportSpecifier") {
        const imported = def.node.imported;
        const importedName = imported?.type === "Literal" ? String(imported.value) : imported?.name;
        if (
          importedName === "argv" &&
          ["process", "node:process"].includes(def.parent?.source?.value)
        ) {
          return ARGUMENTS;
        }
      }
      if (
        def?.type === "Variable" &&
        def.parent?.kind === "const" &&
        def.node?.id?.type === "ObjectPattern"
      ) {
        const key = objectPatternKey(def.node.id, node.name);
        if (key === "argv" && value(def.node.init, depth + 1) === PROCESS) return ARGUMENTS;
        return 0;
      }
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
    if (node.type === "BinaryExpression" && node.operator === "+") {
      return concatenate(classify(node.left, depth + 1), classify(node.right, depth + 1));
    }
    if (
      node.type === "LogicalExpression" &&
      node.operator === "??" &&
      classify(node.right, depth + 1) === ""
    ) {
      return classify(node.left, depth + 1);
    }
    if (node.type === "TemplateLiteral") {
      // Only prefix substitutions may be constant; a suffix substitution is not an argv guard.
      const last = node.expressions.length - 1;
      const argument = classify(node.expressions[last], depth + 1);
      if (identity(argument) !== ARGV || node.quasis[last + 1].value.cooked !== "") return 0;
      let prefix = node.quasis[0].value.cooked;
      for (let i = 0; i < last; i++) {
        const part = classify(node.expressions[i], depth + 1);
        if (typeof part !== "string") return 0;
        prefix = concatenate(prefix, concatenate(part, node.quasis[i + 1].value.cooked));
      }
      return concatenate(prefix, argument);
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
      const target = unwrapExpression(node.object);
      if (
        name === "href" &&
        target.type === "NewExpression" &&
        globalName(target.callee, "URL", context) &&
        target.arguments.length === 1
      ) {
        const argument = classify(target.arguments[0], depth + 1);
        return Array.isArray(argument) && identity(argument) === ARGV ? argument : 0;
      }
      const object = value(node.object, depth + 1);
      if (object === PROCESS && name === "argv") return ARGUMENTS;
      if (object === ARGUMENTS && node.computed && staticIndexOne(node.property)) return ARGV;
      // A URL conversion's href preserves the same module/argv identity.
      if (
        name === "href" &&
        unwrapExpression(node.object).type === "CallExpression" &&
        wrappers.resolveCallTarget(unwrapExpression(node.object))?.calleeName === "pathToFileURL"
      )
        return object;
    }
    if (node.type === "CallExpression") {
      const invoked = logicalCall(node);
      if (!invoked) return 0;
      const callee = unwrapExpression(invoked.callee);
      if (
        callee?.type === "MemberExpression" &&
        memberPropertyName(callee) === "at" &&
        value(callee.object, depth + 1) === ARGUMENTS &&
        invoked.arguments.length === 1 &&
        staticIndexOne(invoked.arguments[0])
      )
        return ARGV;
      if (
        callee?.type === "MemberExpression" &&
        memberPropertyName(callee) === "toString" &&
        invoked.arguments.length === 0
      ) {
        const classified = classify(callee.object, depth + 1);
        return typeof classified === "string" || [ARGV, MAIN].includes(identity(classified))
          ? classified
          : 0;
      }
      if (invoked.arguments.length !== 1 || invoked.arguments[0].type === "SpreadElement") return 0;
      if (globalName(callee, "String", context)) return classify(invoked.arguments[0], depth + 1);
      if (globalName(callee, "Boolean", context) || wrappers.resolveCallTarget(node)) {
        const argument = value(invoked.arguments[0], depth + 1);
        return argument === ARGV || argument === MAIN ? argument : 0;
      }
    }
    return 0;
  }
  return (node) => value(node, 0);
}
function staticIndexOne(node) {
  if (!node) return false;
  if (node.type === "Literal") return node.value === 1 || node.value === "1";
  return literalString(node) === "1";
}

function objectPatternKey(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const value =
      property.value.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value.type !== "Identifier" || value.name !== name) continue;
    if (property.key.type === "Identifier" && !property.computed) return property.key.name;
    if (property.key.type === "Literal") return String(property.key.value);
    return literalString(property.key);
  }
  return null;
}

module.exports = { createClassifier, ARGV, MAIN };
