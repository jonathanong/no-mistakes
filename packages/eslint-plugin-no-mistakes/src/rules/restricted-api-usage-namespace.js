"use strict";

const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");
const { literalString } = require("./logical-call");

function importEqualsSource(node) {
  if (node?.type !== "TSImportEqualsDeclaration" || node.importKind === "type") return null;
  const ref = node.moduleReference;
  if (ref?.type !== "TSExternalModuleReference") return null;
  return literalString(unwrapExpression(ref.expression));
}

function namespaceBinding(expression, context, bindings, seen = new Set()) {
  const object = unwrapExpression(expression);
  if (!object) return null;
  const required = requireSource(object, context);
  if (required) return { source: required, name: null };
  if (object.type !== "Identifier") return null;
  const variable = resolveVariable(object, context);
  if (!variable || seen.has(variable)) return null;
  const known = bindings.get(variable);
  if (known?.name === null) return known;
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  const equals = importEqualsSource(def?.node);
  if (equals) return { source: equals, name: null };
  if (
    def?.type === "Variable" &&
    def.parent?.kind === "const" &&
    def.node?.id?.type === "Identifier"
  ) {
    seen.add(variable);
    return namespaceBinding(def.node.init, context, bindings, seen);
  }
  return null;
}

function matches(target, binding, name) {
  return (
    binding &&
    name !== null &&
    (target.source === undefined || target.source === binding.source) &&
    target.imported?.test(name)
  );
}

function patternKey(property) {
  return memberPropertyName({ property: property.key, computed: property.computed });
}

function patternLocal(property) {
  const value = property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
  return value?.type === "Identifier" ? value : null;
}

function namespaceVisitors({ context, targets, report, bindings, suppressed }) {
  function bindingOf(expression) {
    return namespaceBinding(expression, context, bindings);
  }
  function remember(id, source, name) {
    bindings.set(resolveVariable(id, context), { source, name });
  }
  function sourceLoad(node, source) {
    for (const target of targets) {
      if (target.source === source) {
        report(
          node,
          target,
          source,
          " Use a static named import so the imported API can be checked.",
        );
      }
    }
  }
  function dynamicSource(node, source) {
    const value = literalString(unwrapExpression(source));
    if (value !== null) sourceLoad(node, value);
  }
  function member(node) {
    const binding = bindingOf(node.object);
    const name = memberPropertyName(node);
    if (
      binding &&
      !name &&
      targets.some(
        (target) =>
          target.imported && (target.source === undefined || target.source === binding.source),
      )
    ) {
      if (!suppressed(node)) {
        context.report({ node, messageId: "unanalyzable", data: { name: binding.source } });
      }
      return;
    }
    for (const target of targets) if (matches(target, binding, name)) report(node, target, name);
  }
  function destructure(node) {
    const parent = node.parent;
    const value =
      parent.type === "VariableDeclarator"
        ? parent.init
        : parent.type === "AssignmentExpression"
          ? parent.right
          : null;
    if (!value) return;
    const binding = bindingOf(value);
    for (const property of node.properties) {
      if (property.type === "RestElement" && binding) sourceLoad(property, binding.source);
      if (property.type !== "Property") continue;
      const name = patternKey(property);
      const local = patternLocal(property);
      if (binding && name && local) remember(local, binding.source, name);
      for (const target of targets)
        if (matches(target, binding, name)) report(property, target, name);
    }
  }
  function reexport(node) {
    if (!node.source || node.exportKind === "type") return;
    const binding = { source: node.source.value };
    for (const target of targets) {
      if (node.type === "ExportAllDeclaration") {
        if (target.source === binding.source) report(node, target, binding.source);
        continue;
      }
      for (const specifier of node.specifiers) {
        if (specifier.exportKind === "type") continue;
        const name = specifier.local.name ?? specifier.local.value;
        if (matches(target, binding, name)) report(specifier, target, name);
      }
    }
  }
  function namespaceAlias(node) {
    const value = node.type === "VariableDeclarator" ? node.init : node.right;
    const destination = node.type === "VariableDeclarator" ? node.id : node.left;
    if (!value || destination.type !== "Identifier") return;
    const binding = bindingOf(value);
    if (!binding) return;
    remember(destination, binding.source, null);
    sourceLoad(value, binding.source);
  }
  return {
    dynamicSource,
    visitors: {
      MemberExpression: member,
      ObjectPattern: destructure,
      VariableDeclarator: namespaceAlias,
      AssignmentExpression: namespaceAlias,
      ExportNamedDeclaration: reexport,
      ExportAllDeclaration: reexport,
    },
  };
}

module.exports = { namespaceBinding, matches, namespaceVisitors };
