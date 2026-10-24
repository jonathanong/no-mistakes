"use strict";
const { rule, options } = require("../helpers");
const {
  memberName,
  testBinding,
  isContextObject,
  isContextSkipAlias,
} = require("./test-no-skips-bindings");
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

function annotationText(value) {
  if (value.type === "Literal") return typeof value.value === "string" ? value.value : null;
  if (value.type === "TemplateLiteral" && value.expressions.length === 0)
    return value.quasis[0].value.cooked;
  return null;
}

function isStaticLiteral(value) {
  return (
    value.type === "Literal" || (value.type === "TemplateLiteral" && value.expressions.length === 0)
  );
}

function bannedAnnotation(value, banned) {
  const text = annotationText(value);
  return text && ANNOTATION_TYPES.has(text) && banned.has(text) ? text : null;
}

function annotationTypeName(node, context, banned, visited = new Set()) {
  const value = constInitializer(node, context, visited);
  if (!value?.type) return null;
  if (value.type !== "ConditionalExpression") return bannedAnnotation(value, banned);
  const names = new Set();
  if (!collectBannedAnnotations(value, context, banned, visited, names)) return null;
  return names.size ? [...names].join(" or ") : null;
}

// Every branch must be a literal. Banned annotation names are unique and stay
// in source order. A non-annotation literal does not hide those names, and a
// dynamic branch leaves the whole conditional unresolved.
function collectBannedAnnotations(node, context, banned, visited, names) {
  const value = constInitializer(node, context, visited);
  if (!value?.type) return false;
  if (value.type === "ConditionalExpression") {
    return (
      collectBannedAnnotations(value.consequent, context, banned, new Set(visited), names) &&
      collectBannedAnnotations(value.alternate, context, banned, new Set(visited), names)
    );
  }
  if (!isStaticLiteral(value)) return false;
  const name = bannedAnnotation(value, banned);
  if (name) names.add(name);
  return true;
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
function impliesSkippedDeclaration(node, banned, context) {
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") {
    const name = memberName(current, context);
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
        const modifier = memberName(node, context);
        if (!banned.has(modifier) || suppressed(node.property)) return;
        const test = testBinding(node.object, context);
        const contextCall =
          modifier === "skip" &&
          node.parent.type === "CallExpression" &&
          node.parent.callee === node &&
          isContextObject(node.object, context);
        if (!test && !contextCall) return;
        context.report({ node: node.property, messageId: "modifier", data: { modifier } });
      },
      CallExpression(node) {
        if (
          banned.has("skip") &&
          node.callee.type === "Identifier" &&
          !suppressed(node.callee) &&
          isContextSkipAlias(node.callee, context)
        ) {
          context.report({ node: node.callee, messageId: "modifier", data: { modifier: "skip" } });
        }
        // extend's arguments are fixture options, not test declaration details.
        if (
          node.callee.type === "MemberExpression" &&
          memberName(node.callee, context) === "extend"
        )
          return;
        const binding = testBinding(node.callee, context);
        if (
          binding?.framework !== "@playwright/test" ||
          !["test", "describe"].includes(binding.name)
        )
          return;
        if (impliesSkippedDeclaration(node.callee, banned, context)) return;
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
