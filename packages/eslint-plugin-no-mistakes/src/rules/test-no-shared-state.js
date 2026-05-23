"use strict";

const { rule } = require("../helpers");

const TEST_CALLEES = new Set(["it", "test", "describe"]);

function calleeName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression" && !node.computed) return calleeName(node.object);
  if (node?.type === "CallExpression") return calleeName(node.callee);
  return null;
}

function isTestCall(node) {
  return TEST_CALLEES.has(calleeName(node.callee));
}

function collectPatternNames(node, names = new Set()) {
  if (!node) return names;
  if (node.type === "Identifier") {
    names.add(node.name);
  } else if (node.type === "ObjectPattern") {
    for (const property of node.properties) {
      collectPatternNames(property.value || property.argument, names);
    }
  } else if (node.type === "ArrayPattern") {
    for (const element of node.elements) {
      collectPatternNames(element, names);
    }
  } else if (node.type === "RestElement") {
    collectPatternNames(node.argument, names);
  } else if (node.type === "AssignmentPattern") {
    collectPatternNames(node.left, names);
  }
  return names;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow mutable module-scope test state", recommended: false },
    schema: [],
    messages: {
      shared:
        "Shared mutable module-scope state between tests: use local variables inside each test instead.",
    },
  },
  (context) => {
    const mutableTopLevel = new Set();
    let testDepth = 0;

    function reportIfShared(node, name) {
      if (testDepth > 0 && mutableTopLevel.has(name)) {
        context.report({ node, messageId: "shared" });
      }
    }

    return {
      "Program > VariableDeclaration"(node) {
        if (node.kind === "const") return;
        for (const declaration of node.declarations) {
          for (const name of collectPatternNames(declaration.id)) mutableTopLevel.add(name);
        }
      },
      CallExpression(node) {
        if (isTestCall(node)) testDepth += 1;
      },
      "CallExpression:exit"(node) {
        if (isTestCall(node)) testDepth -= 1;
      },
      AssignmentExpression(node) {
        for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
      },
      UpdateExpression(node) {
        if (node.argument.type === "Identifier") reportIfShared(node, node.argument.name);
      },
    };
  },
);
