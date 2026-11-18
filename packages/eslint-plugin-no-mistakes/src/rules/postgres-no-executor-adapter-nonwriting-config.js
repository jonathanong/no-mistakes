"use strict";

const { stableCallSignature } = require("./postgres-no-executor-adapter-assertion");
const { safeOtherArguments } = require("./postgres-no-executor-adapter-nonwriting-arguments");

const nonWritingCalleesSchema = {
  type: "array",
  items: {
    type: "object",
    properties: {
      module: { type: "string", minLength: 1 },
      export: { type: "string", minLength: 1 },
    },
    required: ["module", "export"],
    additionalProperties: false,
  },
};

function configuredArgument(call, index, checker, ts, facts, program) {
  if (!facts.nonWritingCallees?.length) return false;
  const symbol = checker.getSymbolAtLocation(call.expression);
  const declaration = symbol?.declarations?.[0];
  let imported;
  let importDeclaration;
  if (declaration && ts.isImportSpecifier(declaration)) {
    if (declaration.isTypeOnly || declaration.parent.parent.isTypeOnly) return false;
    imported = (declaration.propertyName ?? declaration.name).text;
    importDeclaration = declaration.parent.parent.parent;
  } else if (declaration && ts.isImportClause(declaration) && !declaration.isTypeOnly) {
    imported = "default";
    importDeclaration = declaration.parent;
  } else return false;
  return (
    facts.nonWritingCallees.some(
      (entry) =>
        entry.module === importDeclaration.moduleSpecifier.text && entry.export === imported,
    ) &&
    Boolean(stableCallSignature(call, checker, ts, facts, program)) &&
    safeOtherArguments(call, index, checker, ts, facts, program)
  );
}

module.exports = { configuredArgument, nonWritingCalleesSchema };
