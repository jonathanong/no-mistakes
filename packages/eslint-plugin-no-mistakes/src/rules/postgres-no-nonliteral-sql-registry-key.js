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
  const entries = new Map();
  for (const property of object.properties) {
    if (property.type !== "Property" || property.kind !== "init" || property.method) return null;
    const key = propertyKey(property);
    const value = stringText(property.value);
    if (key == null || value == null) return null;
    entries.set(key, value);
  }
  return [...entries];
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

function finiteKey(property, object, context) {
  if (property.type !== "TSAsExpression") return false;
  const type = property.typeAnnotation;
  if (type.type !== "TSTypeOperator" || type.operator !== "keyof") return false;
  const query = type.typeAnnotation;
  return (
    query.type === "TSTypeQuery" &&
    query.exprName.type === "Identifier" &&
    resolveVariable(query.exprName, context) === resolveVariable(object, context)
  );
}

function sqlArgument(argument, context) {
  const current = unwrapTs(argument);
  if (current.type !== "Identifier") return current;
  const def = resolveVariable(current, context)?.defs.find(
    (item) => item.node?.type === "VariableDeclarator" && item.node.id.type === "Identifier",
  );
  return def?.parent.kind === "const" ? def.node.init : null;
}

function rewrite(entries, key) {
  const arms = entries.map(
    ([name, value]) => `case ${JSON.stringify(name)}: return ${JSON.stringify(value)};`,
  );
  return `((key) => { switch (key) { ${arms.join(" ")} default: throw new Error("unknown"); } })((${key}))`;
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
    const reported = new WeakSet();
    const bindings = executorBindings(context.sourceCode.ast, options, context.filename, context);

    return {
      CallExpression(node) {
        if (!isDatabaseCall(node, bindings)) return;
        const argument = firstCallArgument(node, bindings);
        if (!argument || suppressed(node)) return;
        visitSql(sqlArgument(argument, context), (piece) => {
          if (
            piece.type !== "MemberExpression" ||
            !piece.computed ||
            stringText(piece.property) != null
          ) {
            return;
          }
          const object = unwrapTs(piece.object);
          if (!object || object.type !== "Identifier" || suppressed(piece) || reported.has(piece))
            return;
          if (finiteKey(piece.property, object, context)) return;
          const entries = registryEntries(object, context);
          if (!entries) return;
          reported.add(piece);
          context.report({
            node: piece,
            messageId: "dynamicKey",
            data: {
              registry: object.name,
              rewrite: rewrite(entries, context.sourceCode.getText(piece.property)),
            },
          });
        });
      },
    };
  },
);
