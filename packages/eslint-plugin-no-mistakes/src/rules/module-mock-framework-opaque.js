"use strict";

function staticImportSource(definition) {
  return require("./async-target-bindings").staticImportSource(definition);
}

function opaqueImportEquals(def) {
  if (def.type !== "ImportBinding" || def.node?.type !== "TSImportEqualsDeclaration") return false;
  const reference = def.node.moduleReference;
  return (
    def.node.importKind !== "type" &&
    reference.type === "TSExternalModuleReference" &&
    !staticImportSource(def)
  );
}

function opaqueIdentifierAlias(def, context, seen, opaque) {
  if (def.type !== "Variable" || def.node?.id?.type !== "Identifier") return false;
  if (def.node.init?.type !== "Identifier") return false;
  return opaque(def.node.init, context, seen);
}

module.exports = { opaqueIdentifierAlias, opaqueImportEquals };
