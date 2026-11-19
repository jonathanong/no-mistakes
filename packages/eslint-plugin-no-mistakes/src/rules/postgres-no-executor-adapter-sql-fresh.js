"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");
const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

function dateType(type, program) {
  const symbol = type.getSymbol();
  return Boolean(
    symbol?.name === "Date" &&
    symbol.declarations?.some((declaration) =>
      program.isSourceFileDefaultLibrary(declaration.getSourceFile()),
    ),
  );
}

// A non-row bind value is a primitive or the built-in Date. Row fields stay primitive-only.
function bindValueType(type, ts, program, allowUndefined) {
  if (type.isUnion())
    return type.types.every((part) => bindValueType(part, ts, program, allowUndefined));
  return primitiveType(type, ts, allowUndefined) || dateType(type, program);
}

// `new Date(...)` holds no row data when every argument is itself a proven scalar.
function freshDate(node, context, scalar) {
  if (node?.type !== "NewExpression") return false;
  const owner = node.parent;
  if (owner.type === "VariableDeclarator" && owner.id.type !== "Identifier") return false;
  const callee = unwrapTs(node.callee);
  if (callee.type !== "Identifier" || callee.name !== "Date") return false;
  const variable = resolveVariable(callee, context);
  return !variable?.defs.length && node.arguments.every((arg) => scalar(arg));
}

// A `const` bound to a fresh Date, unless a later use could write row data into it.
function freshLocal(variable, declarator, reference, context, scalar) {
  const value = declarator.init && unwrapTs(declarator.init);
  if (value?.type !== "NewExpression") return null;
  return (
    freshDate(value, context, scalar) && !objectMutation(variable, null, null, reference, context)
  );
}

module.exports = { bindValueType, freshDate, freshLocal };
