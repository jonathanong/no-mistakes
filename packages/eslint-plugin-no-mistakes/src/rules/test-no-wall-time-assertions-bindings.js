"use strict";

const { staticImportSource } = require("./async-target-bindings");
const { requireSource } = require("./async-target-sources");
const { literalString } = require("./module-mock-helpers");
const { memberName, propertyName, unwrap } = require("./test-message-expect");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { importSpecifierName } = require("./test-no-shared-state-callees");

function constDeclaration(identifier, variable) {
  if (variable?.defs.length !== 1) return null;
  const [definition] = variable.defs;
  const declarator = definition.node;
  return definition.type === "Variable" &&
    definition.parent.kind === "const" &&
    declarator.init &&
    declarator.init.range[1] <= identifier.range[0] &&
    !variable.references.some((reference) => reference.isWrite() && !reference.init)
    ? declarator
    : null;
}

function localOrigin(node, context) {
  let current = unwrap(node);
  while (current?.type === "Identifier") {
    const variable = resolveVariable(current, context);
    const declarator = constDeclaration(current, variable);
    if (
      declarator?.id.type !== "Identifier" ||
      context.sourceCode.getScope(current).variableScope !== variable.scope.variableScope
    ) {
      return current;
    }
    current = unwrap(declarator.init);
  }
  return current;
}

function importNamespaceSource(definition) {
  if (definition?.type !== "ImportBinding") return null;
  if (
    definition.node?.type !== "ImportNamespaceSpecifier" &&
    definition.node?.type !== "TSImportEqualsDeclaration"
  ) {
    return null;
  }
  return staticImportSource(definition);
}

function requiredNamespace(node, context, seen = new Set()) {
  const value = unwrap(node);
  if (!value || seen.has(value)) return null;
  if (value.type !== "Identifier") return requireSource(value, context);
  seen.add(value);
  const variable = resolveVariable(value, context);
  const imported = importNamespaceSource(variable?.defs.length === 1 ? variable.defs[0] : null);
  if (imported) return imported;
  const declaration = constDeclaration(value, variable);
  if (declaration?.id.type !== "Identifier") return null;
  return (
    requiredNamespace(declaration.init, context, seen) || requireSource(declaration.init, context)
  );
}

function requiredMemberAccess(node, context) {
  const name = memberName(node);
  const source = name && requiredNamespace(node.object, context);
  return source ? { name, source } : null;
}

function moduleMember(node, context) {
  if (node?.type !== "Identifier") return requiredMemberAccess(node, context);
  const variable = resolveVariable(node, context);
  const definition = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    definition?.type === "ImportBinding" &&
    definition.node.type === "ImportSpecifier" &&
    definition.node.importKind !== "type" &&
    definition.parent.importKind !== "type"
  ) {
    return {
      name: importSpecifierName(definition.node),
      source: definition.parent.source.value,
    };
  }
  const declaration = constDeclaration(node, variable);
  if (declaration?.id.type === "Identifier") {
    return requiredMemberAccess(unwrap(declaration.init), context);
  }
  if (declaration?.id.type !== "ObjectPattern") return null;
  const source = requiredNamespace(declaration.init, context);
  if (!source) return null;
  const property = declaration.id.properties.find(
    (item) => item.type === "Property" && item.value.name === node.name,
  );
  const name = property?.computed ? literalString(property.key) : propertyName(property?.key);
  return name ? { name, source } : null;
}

module.exports = { localOrigin, moduleMember };
