"use strict";

const { unwrapExpression } = require("./async-ast");
const { compileTargets, matchesAny, targetMatches } = require("./async-patterns");
const { propertyName } = require("./module-mock-helpers");

const LOCAL_REQUIRE_DEFS = new Set(["Parameter", "Variable", "CatchClause", "FunctionName"]);

function findVariable(scope, name) {
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function resolveVariable(node, context) {
  return findVariable(context.sourceCode.getScope(node), node.name);
}

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function requireSource(node, context) {
  const expression = unwrapExpression(node);
  if (
    expression?.type !== "CallExpression" ||
    expression.callee.type !== "Identifier" ||
    expression.callee.name !== "require" ||
    typeof expression.arguments[0]?.value !== "string"
  ) {
    return null;
  }
  const variable = resolveVariable(expression.callee, context);
  if (variable?.defs.some((def) => LOCAL_REQUIRE_DEFS.has(def.type))) return null;
  return expression.arguments[0].value;
}

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function memberPropertyName(node) {
  if (!node.computed) return propertyName(node.property);
  return node.property?.type === "Literal" ? String(node.property.value) : null;
}

function createTargetMatcher(context, optionKey = "targets") {
  const targets = compileTargets(context.options?.[0] || {}, optionKey);
  const sourceSpecifierPatterns = targets.flatMap((target) => target.sourceSpecifierPatterns);
  const directBindings = new Map();
  const namespaceBindings = new Map();

  function recordDirect(id, source, calleeName) {
    if (id?.type !== "Identifier" || !targetMatches(targets, source, calleeName)) return;
    const variable = resolveVariable(id, context);
    if (variable) directBindings.set(variable, { source, calleeName });
  }

  function recordNamespace(id, source) {
    if (id.type !== "Identifier" || !matchesAny(source, sourceSpecifierPatterns)) {
      return;
    }
    const variable = resolveVariable(id, context);
    if (variable) namespaceBindings.set(variable, source);
  }

  function recordRequireDeclarator(node) {
    const source = requireSource(node.init, context);
    if (source) {
      if (node.id.type === "Identifier") {
        recordNamespace(node.id, source);
        recordDirect(node.id, source, node.id.name);
        return;
      }
      if (node.id.type === "ObjectPattern") {
        for (const property of node.id.properties) {
          if (property.type !== "Property") continue;
          recordDirect(bindingIdentifier(property.value), source, propertyName(property.key));
        }
      }
      return;
    }
    const member = unwrapExpression(node.init);
    if (member?.type !== "MemberExpression" || node.id.type !== "Identifier") return;
    const memberSource = requireSource(member.object, context);
    const name = memberPropertyName(member);
    if (memberSource && name) recordDirect(node.id, memberSource, name);
  }

  function recordImportDeclaration(node) {
    const source = node.source.value;
    for (const specifier of node.specifiers) {
      if (specifier.type === "ImportNamespaceSpecifier") {
        recordNamespace(specifier.local, source);
      } else if (specifier.type === "ImportDefaultSpecifier") {
        recordDirect(specifier.local, source, specifier.local.name);
      } else if (specifier.type === "ImportSpecifier") {
        recordDirect(specifier.local, source, importSpecifierName(specifier));
      }
    }
  }

  function recordProgram(node) {
    for (const statement of node.body) {
      if (statement.type === "ImportDeclaration") recordImportDeclaration(statement);
      const declarations =
        statement.type === "VariableDeclaration"
          ? statement.declarations
          : statement.type === "ExportNamedDeclaration" &&
              statement.declaration?.type === "VariableDeclaration"
            ? statement.declaration.declarations
            : [];
      for (const declaration of declarations) recordRequireDeclarator(declaration);
    }
  }

  function resolveDirectTarget(node) {
    if (node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    return (variable && directBindings.get(variable)) || null;
  }

  function resolveNamespaceTarget(node) {
    if (node.type !== "MemberExpression") return null;
    const name = memberPropertyName(node);
    if (!name) return null;
    const object = unwrapExpression(node.object);
    const source =
      requireSource(object, context) ||
      (object.type === "Identifier"
        ? namespaceBindings.get(resolveVariable(object, context))
        : null);
    return source && targetMatches(targets, source, name) ? { source, calleeName: name } : null;
  }

  function resolveCallTarget(node) {
    const callee = unwrapExpression(node.callee);
    return resolveDirectTarget(callee) || resolveNamespaceTarget(callee);
  }

  return {
    hasTargets: targets.length > 0,
    resolveCallTarget,
    isTargetCall(node) {
      return Boolean(resolveCallTarget(node));
    },
    visitors: {
      Program: recordProgram,
      ImportDeclaration: recordImportDeclaration,
      VariableDeclarator: recordRequireDeclarator,
    },
  };
}

module.exports = {
  createTargetMatcher,
  memberPropertyName,
};
