"use strict";
const { resolveVariable } = require("./async-target-bindings");
const importSelectorSchema = {
  type: "object",
  properties: {
    source: { type: "string", minLength: 1 },
    imported: { type: "string", minLength: 1 },
  },
  required: ["source", "imported"],
  additionalProperties: false,
};
function matchesRuntimeImport(id, selector, context) {
  const variable = resolveVariable(id, context);
  return Boolean(
    variable?.defs.some((definition) => {
      const specifier = definition.node;
      const declaration = definition.parent;
      return (
        specifier.type === "ImportSpecifier" &&
        specifier.importKind !== "type" &&
        declaration.importKind !== "type" &&
        declaration.source.value === selector.source &&
        (specifier.imported.name ?? specifier.imported.value) === selector.imported
      );
    }),
  );
}
module.exports = { importSelectorSchema, matchesRuntimeImport };
