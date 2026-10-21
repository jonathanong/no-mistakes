"use strict";
const { rule, options } = require("../helpers");
const { memberName, testBinding, isContextSkip } = require("./test-no-skips-bindings");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const MODIFIERS = ["skip", "skipIf", "runIf", "only", "todo", "fixme"];
const ANNOTATION_TYPES = new Set(["skip", "fixme"]);

function hasMemberWrite(variable) {
  return variable.references.some(({ identifier }) => {
    let current = identifier;
    while (current.parent?.type === "MemberExpression" && current.parent.object === current)
      current = current.parent;
    const parent = current.parent;
    return (
      (parent?.type === "AssignmentExpression" && parent.left === current) ||
      parent?.type === "UpdateExpression" ||
      (parent?.type === "UnaryExpression" && parent.operator === "delete")
    );
  });
}

function constInitializer(node, context, visited) {
  if (!node?.type) return null;
  node = unwrapExpression(node);
  if (!node?.type) return null;
  if (node.type !== "Identifier") return node;
  const binding = resolveVariable(node, context);
  if (!binding || visited.has(binding) || binding.defs.length !== 1) return null;
  const definition = binding.defs[0];
  if (definition.type !== "Variable" || definition.node.parent.kind !== "const") return null;
  if (hasMemberWrite(binding)) return null;
  visited.add(binding);
  return constInitializer(definition.node.init, context, visited);
}

function staticProperty(node, name, context) {
  const value = constInitializer(node, context, new Set());
  if (value?.type !== "ObjectExpression") return null;
  if (value.properties.some((property) => property.type === "SpreadElement" || property.computed))
    return null;
  const matches = value.properties.filter(
    (property) =>
      property.type === "Property" &&
      !property.computed &&
      !property.method &&
      (property.key.name ?? property.key.value) === name,
  );
  return matches.length === 1 ? matches[0].value : null;
}

function annotationTypeName(node, context, banned, visited = new Set()) {
  const value = constInitializer(node, context, visited);
  if (!value?.type) return null;
  if (value.type === "Literal")
    return ANNOTATION_TYPES.has(value.value) && banned.has(value.value) ? value.value : null;
  if (value.type === "TemplateLiteral" && value.expressions.length === 0) {
    const cooked = value.quasis[0].value.cooked;
    return ANNOTATION_TYPES.has(cooked) && banned.has(cooked) ? cooked : null;
  }
  if (value.type !== "ConditionalExpression") return null;
  const whenTrue = annotationTypeName(value.consequent, context, banned, new Set(visited));
  const whenFalse = annotationTypeName(value.alternate, context, banned, new Set(visited));
  if (!whenTrue || !whenFalse) return null;
  // Name every banned type. The consequent alone would call a false
  // `fixme` branch a `skip`.
  return whenTrue === whenFalse ? whenTrue : `${whenTrue} or ${whenFalse}`;
}

function annotationFinding(node, context, banned) {
  const annotations = staticProperty(node, "annotation", context);
  const resolved = constInitializer(annotations || {}, context, new Set());
  const entries = resolved?.type === "ArrayExpression" ? resolved.elements : [resolved];
  for (const entry of entries) {
    if (!entry || entry.type === "SpreadElement") continue;
    const type = staticProperty(entry, "type", context);
    const name = type && annotationTypeName(type, context, banned);
    if (name) return name;
  }
  return null;
}

// `skip` and `fixme` already remove the test. `only` does not, so an annotation
// on the same call is still reported.
function impliesSkippedDeclaration(node, banned) {
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") {
    const name = memberName(current);
    if ((name === "skip" || name === "fixme") && banned.has(name)) return true;
    current = current.object;
  }
  return false;
}
module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "reject skipped, conditional, pending, and focused tests",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          allow: { type: "array", items: { enum: MODIFIERS }, uniqueItems: true },
          testModules: { type: "array", items: { type: "string" }, uniqueItems: true },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      modifier:
        "Test modifier '{{modifier}}' can remove CI coverage. Run the complete test and provide its required environment, or explicitly allow this modifier.",
      annotation:
        "Playwright '{{annotation}}' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "test-no-skips");
    const banned = new Set(
      MODIFIERS.filter((name) => !(options(context).allow || []).includes(name)),
    );
    return {
      MemberExpression(node) {
        const modifier = memberName(node);
        if (!banned.has(modifier) || suppressed(node.property)) return;
        const test = testBinding(node.object, context);
        const contextCall =
          modifier === "skip" &&
          node.parent.type === "CallExpression" &&
          node.parent.callee === node &&
          isContextSkip(node, context);
        if (!test && !contextCall) return;
        context.report({ node: node.property, messageId: "modifier", data: { modifier } });
      },
      CallExpression(node) {
        const binding = testBinding(node.callee, context);
        if (
          binding?.framework !== "@playwright/test" ||
          !["test", "describe"].includes(binding.name)
        )
          return;
        if (impliesSkippedDeclaration(node.callee, banned)) return;
        const details = node.arguments[1];
        if (!details || details.type === "SpreadElement") return;
        const annotation = annotationFinding(details, context, banned);
        if (!annotation || suppressed(details)) return;
        context.report({
          node: details,
          messageId: "annotation",
          data: { annotation },
        });
      },
    };
  },
);
