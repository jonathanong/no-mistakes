"use strict";

const { resolveModuleBinding } = require("./postgres-module-binding");

function importedBinding(identifier, context) {
  const binding = resolveModuleBinding(identifier, context);
  if (!binding || binding.status !== "resolved" || binding.via === "destructure") return null;
  if (binding.kind === "named")
    return { kind: "named", source: binding.source, name: binding.name };
  return { kind: binding.kind, source: binding.source };
}

// `const { sql } = db` is the imported member `db.sql`. A default on the
// property can bind something else, and a computed identifier key is not static.
function destructuredImport(identifier, context) {
  const binding = resolveModuleBinding(identifier, context);
  if (binding?.status !== "resolved" || binding.via !== "destructure") return null;
  if (binding.kind !== "named") return null;
  return { name: binding.name, source: binding.source };
}

module.exports = {
  destructuredImport,
  importedBinding,
};
