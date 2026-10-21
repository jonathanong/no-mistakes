"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  executorBindings,
  executorOptionDefaults,
  firstCallArgument,
  isDatabaseCall,
} = require("./postgres-executor");
const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");

const RULE_ID = "postgres-no-nonliteral-sql-registry-key";

function isFunction(node) {
  return (
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression" ||
    node?.type === "ArrowFunctionExpression"
  );
}

function stringText(node) {
  const value = unwrapTs(node);
  if (value.type === "Literal" && typeof value.value === "string") return value.value;
  if (value.type === "TemplateLiteral" && value.expressions.length === 0) {
    return value.quasis[0].value.cooked;
  }
  return null;
}

function propertyKey(property) {
  if (property.computed) return stringText(property.key);
  if (property.key?.type === "Identifier") return property.key.name;
  return stringText(property.key);
}

function objectEntries(node) {
  const object = unwrapTs(node);
  if (!object || object.type !== "ObjectExpression" || object.properties.length === 0) return null;
  const entries = [];
  for (const property of object.properties) {
    if (property.type !== "Property" || property.kind !== "init" || property.method) return null;
    const key = propertyKey(property);
    const value = stringText(property.value);
    if (key == null || value == null) return null;
    entries.push([key, value]);
  }
  return entries;
}

function registryEntries(identifier, context) {
  const variable = resolveVariable(identifier, context);
  const def = variable?.defs.find(
    (item) => item.node?.type === "VariableDeclarator" && item.node.id?.type === "Identifier",
  );
  if (!def || def.parent.kind !== "const") return null;
  return objectEntries(def.node.init);
}

function visitSql(node, visit) {
  const current = unwrapTs(node);
  if (!current || isFunction(current)) return;
  visit(current);
  for (const child of childNodes(current)) visitSql(child, visit);
}

function rewrite(entries) {
  const arms = entries.map(
    ([key, value]) => `case ${JSON.stringify(key)}: return ${JSON.stringify(value)};`,
  );
  return `${arms.join(" ")} default: throw new Error("unknown");`;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow a non-literal key into a const registry of SQL identifier strings",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: {
          importSpecifier: { type: "string" },
          executorNames: { type: "array", items: { type: "string" } },
        },
      },
    ],
    messages: {
      dynamicKey:
        "{{registry}}[key] can select an inherited property, so this lookup is not a finite SQL identifier. Return each entry from a switch with a throwing default ({{rewrite}}). A runtime own-key check is not judged.",
    },
  },
  (context) => {
    const options = executorOptionDefaults(context.options?.[0] ?? {});
    const suppressed = ruleSuppression(context, RULE_ID);
    const bindings = executorBindings(context.sourceCode.ast, options, context.filename);

    return {
      CallExpression(node) {
        if (!isDatabaseCall(node, bindings)) return;
        const argument = firstCallArgument(node);
        if (!argument) return;
        visitSql(argument, (piece) => {
          if (piece.type !== "MemberExpression" || !piece.computed || stringText(piece.property)) {
            return;
          }
          const object = unwrapTs(piece.object);
          if (!object || object.type !== "Identifier" || suppressed(piece)) return;
          const entries = registryEntries(object, context);
          if (!entries) return;
          context.report({
            node: piece,
            messageId: "dynamicKey",
            data: { registry: object.name, rewrite: rewrite(entries) },
          });
        });
      },
    };
  },
);
