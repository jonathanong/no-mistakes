"use strict";

const { rule } = require("../helpers");

const TEST_CALLEES = new Set(["it", "test", "describe"]);
const MUTATING_METHODS = new Set(
  "add clear copyWithin delete fill pop push reverse set shift sort splice unshift".split(" "),
);
const MUTABLE_CONSTRUCTORS = new Set(["Map", "Set", "WeakMap", "WeakSet"]);

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
    return names;
  }
  const children =
    node.type === "ObjectPattern"
      ? node.properties.map((property) => property.value || property.argument)
      : node.type === "ArrayPattern"
        ? node.elements
        : node.type === "RestElement"
          ? [node.argument]
          : node.type === "AssignmentPattern"
            ? [node.left]
            : [];
  for (const child of children) collectPatternNames(child, names);
  return names;
}

function mutationRootName(node) {
  if (node.type === "Identifier") return node.name;
  if (node.type === "MemberExpression") return mutationRootName(node.object);
  return null;
}

function propertyName(node) {
  if (node.type === "Literal") return String(node.value);
  return node.name;
}

function mutatingCallRootName(node) {
  if (node.callee.type !== "MemberExpression") return null;
  if (!MUTATING_METHODS.has(propertyName(node.callee.property))) return null;
  return mutationRootName(node.callee.object);
}

function isFunctionNode(node) {
  return ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(
    node.type,
  );
}

function isInlineTestCallback(node) {
  return node.parent?.type === "CallExpression" && isTestCall(node.parent);
}

function isCalledFunction(node) {
  return node.parent?.type === "CallExpression" && node.parent.callee === node;
}

function isMutableInitializer(node) {
  if (!node) return false;
  if (node.type === "ArrayExpression" || node.type === "ObjectExpression") return true;
  return (
    node.type === "NewExpression" &&
    node.callee.type === "Identifier" &&
    MUTABLE_CONSTRUCTORS.has(node.callee.name)
  );
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

    function isModuleMutable(node, name) {
      let scope = context.sourceCode.getScope(node);
      while (scope) {
        const variable = scope.variables.find((candidate) => candidate.name === name);
        if (variable) {
          return (
            mutableTopLevel.has(variable.name) &&
            (variable.scope.type === "module" || variable.scope.block.type === "Program")
          );
        }
        scope = scope.upper;
      }
      return false;
    }

    function reportIfShared(node, name) {
      if (testDepth > 0 && isModuleMutable(node, name)) {
        context.report({ node, messageId: "shared" });
      }
    }
    function reportAssignment(node) {
      for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
      if (node.left.type !== "MemberExpression") return;
      const rootName = mutationRootName(node.left);
      if (rootName) reportIfShared(node, rootName);
    }
    function reportRootMutation(node, rootName) {
      if (rootName) reportIfShared(node, rootName);
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
        for (const declaration of node.declarations) {
          if (node.kind === "const" && !isMutableInitializer(declaration.init)) continue;
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
      AssignmentExpression(node) {
        if (isInsideUncalledNestedFunction(node)) return;
        reportAssignment(node);
      },
      UpdateExpression(node) {
        if (isInsideUncalledNestedFunction(node)) return;
        reportRootMutation(node, mutationRootName(node.argument));
      },
      "CallExpression:exit"(node) {
        if (isTestCall(node)) testDepth -= 1;
        if (isInsideUncalledNestedFunction(node)) return;
        reportRootMutation(node, mutatingCallRootName(node));
      },
    };

    function isInsideUncalledNestedFunction(node) {
      if (testDepth === 0) return false;
      let current = node.parent;
      while (current) {
        const isUncalledFunction =
          isFunctionNode(current) && !isInlineTestCallback(current) && !isCalledFunction(current);
        if (isUncalledFunction) {
          return true;
        }
        current = current.parent;
      }
      return false;
    }

    function checkSharedMutations(node) {
      if (isFunctionNode(node) && !isCalledFunction(node)) return;
      if (node.type === "AssignmentExpression") {
        reportAssignment(node);
      } else if (node.type === "UpdateExpression") {
        reportRootMutation(node, mutationRootName(node.argument));
      } else if (node.type === "CallExpression") {
        reportRootMutation(node, mutatingCallRootName(node));
      }
      for (const child of childNodes(node)) checkSharedMutations(child);
    }
  },
);
