"use strict";

function isCursorModule(name, config) {
  return typeof name === "string" && config.modules.has(name);
}

function isCursorExecutor(name, config) {
  return typeof name === "string" && config.executors.has(name);
}

function importDefinition(variable, specifierType, config) {
  return variable?.defs.find((definition) => {
    const specifier = definition.node;
    const declaration = definition.parent || specifier.parent;
    return (
      definition.type === "ImportBinding" &&
      specifier.type === specifierType &&
      specifier.importKind !== "type" &&
      declaration?.type === "ImportDeclaration" &&
      declaration.importKind !== "type" &&
      isCursorModule(declaration.source?.value, config)
    );
  });
}

function namedCursorImport(context, identifier, helpers, config) {
  if (identifier?.type !== "Identifier") return null;
  const imported = importDefinition(
    helpers.findVariable(context, identifier),
    "ImportSpecifier",
    config,
  )?.node.imported;
  const name = imported?.name ?? imported?.value;
  return isCursorExecutor(name, config) ? String(name) : null;
}

function namespaceCursorImport(context, identifier, helpers, config) {
  return (
    identifier?.type === "Identifier" &&
    Boolean(
      importDefinition(
        helpers.findVariable(context, identifier),
        "ImportNamespaceSpecifier",
        config,
      ),
    )
  );
}

function namespaceImportMember(context, node, helpers, config) {
  const member = helpers.unwrap(node);
  if (member?.type !== "MemberExpression") return null;
  const object = helpers.unwrap(member.object);
  if (object?.type !== "Identifier") return null;
  return namespaceCursorImport(context, object, helpers, config) ? member : null;
}

function namespaceCursorMember(context, node, helpers, config) {
  const member = namespaceImportMember(context, node, helpers, config);
  const name = member && helpers.propertyName(member);
  return isCursorExecutor(name, config) ? String(name) : null;
}

module.exports = {
  isCursorExecutor,
  isCursorModule,
  namedCursorImport,
  namespaceCursorImport,
  namespaceCursorMember,
  namespaceImportMember,
};
