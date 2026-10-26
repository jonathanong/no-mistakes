"use strict";
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const {
  resolveVariable,
  memberPropertyName,
  stableConstInitializer,
} = require("./async-target-bindings");
const { globalObserverMutated } = require("./async-observer-mutations");

const CLOSED = "\0closed-data";
const prototypeKeys = new Set([
  "__proto__",
  "prototype",
  "constructor",
  "toString",
  "toLocaleString",
  "valueOf",
  "hasOwnProperty",
  "isPrototypeOf",
  "propertyIsEnumerable",
  "__defineGetter__",
  "__defineSetter__",
  "__lookupGetter__",
  "__lookupSetter__",
]);

// These facts describe source values, never the structural types of arbitrary holders.
function createDataFacts() {
  const environments = new WeakMap();
  function environment(context, name) {
    let facts = environments.get(context.sourceCode);
    if (!facts) environments.set(context.sourceCode, (facts = new Map()));
    if (!facts.has(name)) facts.set(name, !globalObserverMutated(name, context));
    return facts.get(name);
  }
  function builtin(id, name, context) {
    return (
      id?.type === "Identifier" &&
      id.name === name &&
      !resolveVariable(id, context)?.defs.length &&
      environment(context, name)
    );
  }
  function primitive(node, context, seen = new Set(), snapshot = false) {
    node = unwrapExpression(node);
    if (!node || seen.has(node) || seen.size >= 32) return false;
    seen = new Set(seen).add(node);
    if (node.type === "Literal") return node.value === null || typeof node.value !== "object";
    if (node.type === "UnaryExpression" && ["+", "-", "!", "~", "void"].includes(node.operator))
      return primitive(node.argument, context, seen, snapshot);
    if (node.type === "BinaryExpression")
      return (
        primitive(node.left, context, seen, snapshot) &&
        primitive(node.right, context, seen, snapshot)
      );
    if (
      node.type === "CallExpression" &&
      !node.optional &&
      node.callee.type === "MemberExpression" &&
      builtin(node.callee.object, "Date", context) &&
      ["UTC", "parse"].includes(memberPropertyName(node.callee))
    ) {
      // An already evaluated immutable binding holds the builtin's numeric result.
      // Direct argument evaluation still has to exclude object coercion/getters.
      return snapshot || node.arguments.every((arg) => primitive(arg, context, seen));
    }
    if (node.type !== "Identifier") return false;
    const init = stableConstInitializer(node, context, new Set());
    if (init) return primitive(init, context, seen, true);
    const binding = resolveVariable(node, context);
    if (binding?.defs.length !== 1 || binding.references.some((ref) => ref.isWrite() && !ref.init))
      return false;
    const def = binding.defs[0],
      declaration = def.node,
      loop = def.parent?.parent;
    if (
      def.type !== "Variable" ||
      def.parent.kind !== "const" ||
      loop?.type !== "ForOfStatement" ||
      loop.left !== def.parent ||
      declaration.id.type !== "ArrayPattern"
    )
      return false;
    const index = declaration.id.elements.indexOf(def.name);
    const rows = unwrapExpression(loop.right);
    return (
      environment(context, "Array") &&
      index >= 0 &&
      rows.type === "ArrayExpression" &&
      rows.elements.length > 0 &&
      rows.elements.every((row) => {
        row = unwrapExpression(row);
        return (
          row?.type === "ArrayExpression" && primitive(row.elements[index], context, seen, true)
        );
      })
    );
  }
  function date(node, context) {
    node = unwrapExpression(node);
    return (
      node?.type === "NewExpression" &&
      builtin(node.callee, "Date", context) &&
      node.arguments.length <= 7 &&
      node.arguments.every((arg) => primitive(arg, context))
    );
  }
  function closed(object, context, inherited) {
    return (
      environment(context, "Object") &&
      object.properties.every((entry) =>
        entry.type === "SpreadElement"
          ? inherited.get(CLOSED) === true
          : entry.type === "Property" &&
            !entry.computed &&
            !entry.method &&
            entry.kind === "init" &&
            entry.key.name !== CLOSED &&
            entry.key.value !== CLOSED &&
            !["__proto__", "constructor", "prototype"].includes(entry.key.name ?? entry.key.value),
      )
    );
  }
  function safeRead(member, context, shape) {
    const name = memberPropertyName(member);
    // Missing data is undefined: even a later throwing member read cannot invoke
    // another supplied callback. Prototype properties and mutations are excluded.
    if (
      shape.get(CLOSED) === true &&
      !shape.has(name) &&
      !prototypeKeys.has(name) &&
      environment(context, "Object")
    )
      return true;
    if (shape.get(name) !== "date" || !environment(context, "Date")) return false;
    function read(value) {
      value = unwrapTransparentParent(value);
      const parent = value.parent;
      if (parent?.type === "LogicalExpression" && parent.operator === "??" && parent.left === value)
        return date(parent.right, context) && read(parent);
      if (
        parent?.type === "MemberExpression" &&
        parent.object === value &&
        !parent.optional &&
        ["getTime", "toISOString"].includes(memberPropertyName(parent))
      ) {
        const call = parent.parent;
        return (
          call?.type === "CallExpression" &&
          call.callee === parent &&
          !call.optional &&
          !call.arguments.length
        );
      }
      return false;
    }
    return read(member);
  }
  return { closed, date, safeRead };
}

module.exports = { CLOSED, createDataFacts };
