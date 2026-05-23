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

function mutationRootName(node) {
  if (node.type === "Identifier") return node.name;
  if (node.type === "MemberExpression") return mutationRootName(node.object);
  return null;
}

function childNodes(node) {
  return Object.entries(node)
    .filter(([key]) => !["parent", "loc", "range", "tokens", "comments"].includes(key))
    .flatMap(([, value]) => (Array.isArray(value) ? value : [value]))
    .filter((value) => value && typeof value.type === "string");
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
    const functionDeclarations = new Map();
    let testDepth = 0;

    function reportIfShared(node, name) {
      if (testDepth > 0 && mutableTopLevel.has(name)) {
        context.report({ node, messageId: "shared" });
      }
    }

    return {
      "Program > VariableDeclaration"(node) {
        for (const declaration of node.declarations) {
          if (
            declaration.id.type === "Identifier" &&
            (declaration.init?.type === "FunctionExpression" ||
              declaration.init?.type === "ArrowFunctionExpression")
          ) {
            functionDeclarations.set(declaration.id.name, declaration.init);
          }
        }
        if (node.kind === "const") return;
        for (const declaration of node.declarations) {
          for (const name of collectPatternNames(declaration.id)) mutableTopLevel.add(name);
        }
      },
      "Program > FunctionDeclaration"(node) {
        if (node.id?.name) functionDeclarations.set(node.id.name, node);
      },
      CallExpression(node) {
        if (isTestCall(node)) {
          testDepth += 1;
          const callback = node.arguments.find((argument) => argument.type === "Identifier");
          const declaration = callback ? functionDeclarations.get(callback.name) : null;
          if (declaration) {
            checkSharedMutations(declaration.body);
          }
        }
      },
      "CallExpression:exit"(node) {
        if (isTestCall(node)) testDepth -= 1;
      },
      AssignmentExpression(node) {
        for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
        if (node.left.type === "MemberExpression") {
          const rootName = mutationRootName(node.left);
          if (rootName) reportIfShared(node, rootName);
        }
      },
      UpdateExpression(node) {
        const rootName = mutationRootName(node.argument);
        if (rootName) reportIfShared(node, rootName);
      },
    };

    function checkSharedMutations(node) {
      if (node.type === "AssignmentExpression") {
        for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
        if (node.left.type === "MemberExpression") {
          const rootName = mutationRootName(node.left);
          if (rootName) reportIfShared(node, rootName);
        }
      } else if (node.type === "UpdateExpression") {
        const rootName = mutationRootName(node.argument);
        if (rootName) reportIfShared(node, rootName);
      }
      for (const child of childNodes(node)) checkSharedMutations(child);
    }
  },
);
