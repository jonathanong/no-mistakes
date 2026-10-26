"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function importedBinding(identifier, context) {
  if (identifier?.type !== "Identifier") return null;
  const variable = resolveVariable(identifier, context);
  const definition = variable?.defs.find(
    (item) => item.type === "ImportBinding" && item.node.importKind !== "type",
  );
  const specifier = definition?.node;
  const declaration = specifier?.parent;
  if (
    !specifier ||
    declaration?.type !== "ImportDeclaration" ||
    declaration.importKind === "type" ||
    typeof declaration.source?.value !== "string"
  ) {
    return null;
  }
  if (specifier.type === "ImportNamespaceSpecifier") {
    return { kind: "namespace", source: declaration.source.value };
  }
  if (specifier.type !== "ImportSpecifier") return null;
  const imported = specifier.imported;
  const name = imported.type === "Identifier" ? imported.name : String(imported.value);
  return { kind: "named", source: declaration.source.value, name };
}

// `const { sql } = db` is the namespace member `db.sql`. A default on the
// property can bind something else, and a computed identifier key is not static.
function destructuredImport(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable) return null;
  if (variable.defs.length !== 1) return null;
  const definition = variable.defs[0];
  const declarator = definition.node;
  if (definition.type !== "Variable") return null;
  if (definition.parent.kind !== "const") return null;
  if (declarator.id.type !== "ObjectPattern") return null;
  const property = declarator.id.properties.find(
    (item) => item.type === "Property" && item.value === definition.name,
  );
  if (!property) return null;
  const key = staticKeyName(property);
  if (!key) return null;
  const object = unwrapExpression(declarator.init);
  if (object.type !== "Identifier") return null;
  const imported = importedBinding(object, context);
  if (!imported) return null;
  if (imported.kind !== "namespace") return null;
  return { name: key, source: imported.source };
}

module.exports = {
  destructuredImport,
  importedBinding,
};
