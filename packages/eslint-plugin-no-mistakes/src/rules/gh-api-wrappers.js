"use strict";
const { unwrapSequence } = require("./async-ast");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");
const { propertyName, logicalCall } = require("./module-mock-helpers");

function wrapperName(node, context, roots, seen = new Set()) {
  node = unwrapSequence(node);
  if (node?.type === "MemberExpression") {
    const name = memberPropertyName(node);
    const object = name && wrapperName(node.object, context, roots, seen);
    return object ? `${object}.${name}` : null;
  }
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (seen.has(variable) || variable?.references.some((ref) => ref.isWrite() && !ref.init))
    return null;
  const moduleRoot = !variable || ["module", "global"].includes(variable.scope.type);
  if (moduleRoot && roots.has(node.name)) return node.name;
  if (!variable || variable.defs.length !== 1) return null;
  seen.add(variable);
  const def = variable.defs[0];
  if (def.type !== "Variable" || def.parent?.kind !== "const" || !def.node.init) return null;
  if (def.node.id.type === "Identifier") return wrapperName(def.node.init, context, roots, seen);
  if (def.node.id.type !== "ObjectPattern") return null;
  const property = def.node.id.properties.find(
    (item) =>
      item.type === "Property" && item.value.type === "Identifier" && item.value.name === node.name,
  );
  if (!property) return null;
  const name = property.computed
    ? memberPropertyName({ computed: true, property: property.key })
    : propertyName(property.key);
  const object = name && wrapperName(def.node.init, context, roots, seen);
  return object ? `${object}.${name}` : null;
}

function createWrapperResolver(context, wrappers) {
  const roots = new Set(wrappers.map((entry) => entry.callee.split(".")[0]));
  function resolve(callee, args) {
    const name = wrapperName(callee, context, roots);
    const wrapper = wrappers.find((entry) => entry.callee === name);
    return wrapper ? { wrapper, arguments: args } : null;
  }
  return (node) => {
    // A configured method literally named call/apply owns its original arguments.
    const direct = resolve(node.callee, node.arguments);
    if (direct) return direct;
    const invoked = logicalCall(node);
    return invoked ? resolve(invoked.callee, invoked.arguments) : null;
  };
}
module.exports = { createWrapperResolver };
